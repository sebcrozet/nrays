#![warn(non_camel_case_types)]

extern crate png;
extern crate alga;
extern crate nalgebra as na;
extern crate ncollide;
extern crate nrays;

use std::io::Read;
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::fs::File;
use std::mem;
use std::str::SplitWhitespace;
use std::collections::HashMap;
use std::sync::Arc;
use alga::linear::ProjectiveTransformation;
use na::{Point2, Point3, Vector2, Vector3, Isometry3, Perspective3};
use ncollide::bounding_volume::{AABB, HasBoundingVolume, support_map_aabb};
use ncollide::shape::{SupportMap, Plane, Ball, Cone, Cylinder, Capsule, Cuboid, TriMesh};
use ncollide::query::{Ray3, RayCast, RayIntersection3};
use ncollide::query::ray_internal;
use ncollide::query::algorithms::johnson_simplex::JohnsonSimplex;
use nrays::scene_node::SceneNode;
use nrays::material::Material;
use nrays::normal_material::NormalMaterial;
use nrays::phong_material::PhongMaterial;
use nrays::texture2d::{Texture2d, Interpolation, Overflow};
use nrays::uv_material::UVMaterial;
use nrays::scene::Scene;
use nrays::scene;
use nrays::light::Light;
use nrays::obj;
use nrays::mtl;
use nrays::math::{Point, Vect, Matrix};

fn main() {
    let mut args = env::args();

    if args.len() != 2 {
        panic!("Usage: {} scene_file", args.next().unwrap());
    }

    let exname = args.next().unwrap();
    let path   = args.next().expect(&format!("Usage: {} scene_file", exname));
    let path   = Path::new(&path[..]);

    println!("Loading the scene.");
    let file  = File::open(&path);

    if file.is_err() {
        panic!("Unable to find the file: {}", path.to_str().unwrap());
    }

    let mut file  = file.unwrap();
    let mut descr = String::new();

    let _ = file.read_to_string(&mut descr);

    let (lights, nodes, cameras) = parse(&descr[..]);
    let nnodes    = nodes.len();
    let nlights   = lights.len();
    let ncams     = cameras.len();
    let scene = Arc::new(Scene::new(nodes, lights, Vector3::from_element(1.0)));
    println!("Scene loaded. {} lights, {} objects, {} cameras.", nlights, nnodes, ncams);

    for c in cameras.into_iter() {
        let perspective = Perspective3::new(
            c.resolution.x / c.resolution.y,
            c.fovy.to_radians(),
            1.0,
            100000.0).unwrap();

        let camera = Isometry3::<f64>::look_at_rh(&c.eye, &c.at, &Vector3::y());

        let projection = (perspective * camera.to_homogeneous()).try_inverse().unwrap();

        println!("Casting {} rays per pixels (win. {}).", c.aa.x as usize, c.aa.y);

        let pixels = scene::render(&scene, &c.resolution, c.aa.x as usize, c.aa.y, c.eye, projection);

        println!("Rays cast.");

        println!("Saving image to: {}", c.output);
        pixels.to_png(&Path::new(&c.output[..]));
        println!("Image saved.");
    }
}

//
//
//
//  Scene file parser
//
//
//
enum Mode {
    LightMode,
    ShapeMode,
    CameraMode,
    NoMode
}

#[derive(Clone)]
enum Shape {
    GBall(f64),
    GPlane(Vector3<f64>),
    GCuboid(Vector3<f64>),
    GCylinder(f64, f64),
    GCapsule(f64, f64),
    GCone(f64, f64),
    GObj(String, String),
}

struct Camera {
    eye:        Point3<f64>,
    at:         Point3<f64>,
    fovy:       f64,
    resolution: Vector2<f64>,
    aa:         Vector2<f64>,
    output:     String
}

impl Camera {
    pub fn new(eye: Point3<f64>, at: Point3<f64>, fovy: f64, resolution: Vector2<f64>, aa: Vector2<f64>, output: String) -> Camera {
        assert!(aa.x >= 1.0, "The number of ray per pixel must be at least 1.0");

        Camera {
            eye:        eye,
            at:         at,
            fovy:       fovy,
            resolution: resolution,
            aa:         aa,
            output:     output
        }
    }
}

struct Properties {
    superbloc:  usize,
    geom:       Vec<(usize, Shape)>,
    pos:        Option<(usize, Point3<f64>)>,
    angle:      Option<(usize, Vector3<f64>)>,
    material:   Option<(usize, String)>,
    eye:        Option<(usize, Point3<f64>)>,
    at:         Option<(usize, Point3<f64>)>,
    fovy:       Option<(usize, f64)>,
    color:      Option<(usize, Point3<f64>)>,
    resolution: Option<(usize, Vector2<f64>)>,
    output:     Option<(usize, String)>,
    refl:       Option<(usize, Vector2<f64>)>,
    refr:       Option<(usize, f64)>,
    aa:         Option<(usize, Vector2<f64>)>,
    radius:     Option<(usize, f64)>,
    nsample:    Option<(usize, f64)>,
    solid:      bool,
    flat:       bool,
}

impl Properties {
    pub fn new(l: usize) -> Properties {
        Properties {
            superbloc:  l,
            geom:       Vec::new(),
            pos:        None,
            angle:      None,
            material:   None,
            eye:        None,
            at:         None,
            fovy:       None,
            color:      None,
            resolution: None,
            output:     None,
            refl:       None,
            refr:       None,
            aa:         None,
            radius:     None,
            nsample:    None,
            solid:      false,
            flat:       false,
        }
    }
}

fn error(line: usize, err: &str) -> ! {
    panic!("At line {}: {}", line, err)
}

fn warn(line: usize, err: &str) {
    println!("At line {}: {}", line, err)
}

fn parse(string: &str) -> (Vec<Light>, Vec<Arc<SceneNode>>, Vec<Camera>) {
    let mut nodes   = Vec::new();
    let mut lights  = Vec::new();
    let mut cameras = Vec::new();
    let mut props   = Properties::new(0);
    let mut mode    = Mode::NoMode;
    let mut mtllib  = HashMap::new();

    for (l, line) in string.lines().enumerate() {
        let mut words  = line.split_whitespace();
        let tag        = words.next();

        let white = Arc::new(Box::new(PhongMaterial::new(
            Point3::new(0.1, 0.1, 0.1),
            Point3::new(1.0, 1.0, 1.0),
            Point3::new(1.0, 1.0, 1.0),
            None,
            None,
            100.0
        )) as Box<Material + 'static + Send + Sync>);

        mtllib.insert("normals".to_string(), (1.0, Arc::new(Box::new(NormalMaterial::new()) as Box<Material + 'static + Send + Sync>)));
        mtllib.insert("uvs".to_string(), (1.0, Arc::new(Box::new(UVMaterial::new()) as Box<Material + 'static + Send + Sync>)));
        mtllib.insert("default".to_string(), (1.0, white));

        match tag {
            None    => { },
            Some(w) => {
                if w.len() != 0 && w.as_bytes()[0] != ('#' as u8) {
                    match w {
                        // top-level commands
                        "mtllib"   => register_mtllib(&parse_name(l, words)[..], &mut mtllib),
                        "light"    => {
                            let old = mem::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = Mode::LightMode;
                        },
                        "geometry" => {
                            let old = mem::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = Mode::ShapeMode;
                        },
                        "camera"   => {
                            let old = mem::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = Mode::CameraMode;
                        },
                        // common attributes
                        "color"      => props.color      = Some((l, Point3::from_coordinates(parse_triplet(l, words)))),
                        "angle"      => props.angle      = Some((l, parse_triplet(l, words))),
                        "pos"        => props.pos        = Some((l, Point3::from_coordinates(parse_triplet(l, words)))),
                        "eye"        => props.eye        = Some((l, Point3::from_coordinates(parse_triplet(l, words)))),
                        "at"         => props.at         = Some((l, Point3::from_coordinates(parse_triplet(l, words)))),
                        "material"   => props.material   = Some((l, parse_name(l, words))),
                        "fovy"       => props.fovy       = Some((l, parse_number(l, words))),
                        "output"     => props.output     = Some((l, parse_name(l, words))),
                        "resolution" => props.resolution = Some((l, parse_duet(l, words))),
                        "refl"       => props.refl       = Some((l, parse_duet(l, words))),
                        "refr"       => props.refr       = Some((l, parse_number(l, words))),
                        "aa"         => props.aa         = Some((l, parse_duet(l, words))),
                        "radius"     => props.radius     = Some((l, parse_number(l, words))),
                        "nsample"    => props.nsample    = Some((l, parse_number(l, words))),
                        // geometries
                        "ball"       => props.geom.push((l, parse_ball(l, words))),
                        "plane"      => props.geom.push((l, parse_plane(l, words))),
                        "box"        => props.geom.push((l, parse_box(l, words))),
                        "cylinder"   => props.geom.push((l, parse_cylinder(l, words))),
                        "capsule"    => props.geom.push((l, parse_capsule(l, words))),
                        "cone"       => props.geom.push((l, parse_cone(l, words))),
                        "obj"        => props.geom.push((l, parse_obj(l, words))),
                        "solid"      => props.solid = true,
                        "flat"       => props.flat = true,
                        _             => {
                            println!("Warning: unknown line {} ignored: `{}'", l, line);
                        }
                    }
                }
            }
        }
    }

    register(&mode, props, &mut mtllib, &mut lights, &mut nodes, &mut cameras);

    (lights, nodes, cameras)
}

fn register(mode:    &Mode,
            props:   Properties,
            mtllib:  &mut HashMap<String, (f32, Arc<Box<Material + 'static + Send + Sync>>)>,
            lights:  &mut Vec<Light>,
            nodes:   &mut Vec<Arc<SceneNode>>,
            cameras: &mut Vec<Camera>) {
    match *mode {
        Mode::LightMode  => register_light(props, lights),
        Mode::ShapeMode   => register_geometry(props, mtllib, nodes),
        Mode::CameraMode => register_camera(props, cameras),
        Mode::NoMode     => register_nothing(props),
    }
}

fn warn_if_not_empty<T>(t: &[(usize, T)]) {
    if !t.is_empty() {
        for g in t.iter() {
            warn(g.0.clone(), "dropped unexpected attribute.")
        }
    }
}

fn warn_if_some<T>(t: &Option<(usize, T)>) {
    t.as_ref().map(|&(l, _)| warn(l, "dropped unexpected attribute."));
}

fn fail_if_empty<T>(t: &[(usize, T)], l: usize, attrname: &str) {
    if t.is_empty() {
        error(l, &format!("missing attribute: {}", attrname)[..]);
    }
}

fn fail_if_none<T>(t: &Option<(usize, T)>, l: usize, attrname: &str) {
    match *t {
        None    => error(l, &format!("missing attribute: {}", attrname)[..]),
        Some(_) => { }
    }
}

fn register_nothing(props: Properties) {
    warn_if_not_empty(&props.geom[..]);
    warn_if_some(&props.pos);
    warn_if_some(&props.angle);
    warn_if_some(&props.material);
    warn_if_some(&props.eye);
    warn_if_some(&props.at);
    warn_if_some(&props.fovy);
    warn_if_some(&props.color);
    warn_if_some(&props.output);
    warn_if_some(&props.resolution);
    warn_if_some(&props.refl);
    warn_if_some(&props.refr);
    warn_if_some(&props.aa);
    warn_if_some(&props.radius);
    warn_if_some(&props.nsample);
}

fn register_camera(props: Properties, cameras: &mut Vec<Camera>) {
    warn_if_not_empty(&props.geom[..]);
    warn_if_some(&props.pos);
    warn_if_some(&props.angle);
    warn_if_some(&props.material);
    warn_if_some(&props.refl);
    warn_if_some(&props.refr);
    warn_if_some(&props.radius);
    warn_if_some(&props.nsample);

    let l = props.superbloc;

    fail_if_none(&props.output, l, "output <filename>");
    fail_if_none(&props.resolution, l, "resolution <x> <y>");
    fail_if_none(&props.eye, l, "eye <x> <y> <z>");
    fail_if_none(&props.at, l, "at <x> <y> <z>");
    fail_if_none(&props.fovy, l, "fovy <value>");


    let aa   = props.aa.unwrap_or((l, Vector2::new(1.0, 0.0)));
    let eye  = props.eye.unwrap().1;
    let at   = props.at.unwrap().1;
    let fov  = props.fovy.unwrap().1;
    let res  = props.resolution.unwrap().1;
    let name = props.output.unwrap().1;

    cameras.push(Camera::new(eye, at, fov, res, aa.1, name));
}

fn register_light(props: Properties, lights: &mut Vec<Light>) {
    warn_if_not_empty(&props.geom[..]);
    warn_if_some(&props.angle);
    warn_if_some(&props.material);
    warn_if_some(&props.eye);
    warn_if_some(&props.at);
    warn_if_some(&props.fovy);
    warn_if_some(&props.output);
    warn_if_some(&props.resolution);
    warn_if_some(&props.refl);
    warn_if_some(&props.refr);
    warn_if_some(&props.aa);

    fail_if_none(&props.pos, props.superbloc, "pos <x> <y> <z>");
    fail_if_none(&props.color, props.superbloc, "color <r> <g> <b>");

    let radius  = props.radius.unwrap_or((props.superbloc, 0.0)).1;
    let nsample = props.nsample.unwrap_or((props.superbloc, 1.0)).1;
    let pos     = props.pos.unwrap().1;
    let color: Point3<f32> = na::convert(props.color.unwrap().1);
    let light   = Light::new(pos, radius, nsample as usize, color);

    lights.push(light);
}

fn register_mtllib(path: &str, mtllib: &mut HashMap<String, (f32, Arc<Box<Material + 'static + Send + Sync>>)>) {
    let materials = mtl::parse_file(&Path::new(path)).unwrap(); // expect(format!("Failed to parse the mtl file: {}", path));

    for m in materials.into_iter() {
        let t = m.diffuse_texture.as_ref().map(|t| Texture2d::from_png(&Path::new(&t[..]), false, Interpolation::Bilinear, Overflow::Wrap).expect("Image not found."));

        let a = m.opacity_map.as_ref().map(|t| Texture2d::from_png(&Path::new(&t[..]), true, Interpolation::Bilinear, Overflow::Wrap).expect("Image not found."));

        let alpha = m.alpha;

        let color = Box::new(PhongMaterial::new(
            m.ambiant,
            m.diffuse,
            m.specular,
            t,
            a,
            m.shininess
            )) as Box<Material + 'static + Send + Sync>;

        mtllib.insert(m.name, (alpha, Arc::new(color)));
    }
}

fn register_geometry(props:  Properties,
                     mtllib: &mut HashMap<String, (f32, Arc<Box<Material + 'static + Send + Sync>>)>,
                     nodes:  &mut Vec<Arc<SceneNode>>) {
    warn_if_some(&props.eye);
    warn_if_some(&props.at);
    warn_if_some(&props.fovy);
    warn_if_some(&props.color);
    warn_if_some(&props.output);
    warn_if_some(&props.resolution);

    fail_if_none(&props.pos, props.superbloc, "pos <x> <y> <z>");
    fail_if_none(&props.angle, props.superbloc, "color <r> <g> <b>");
    fail_if_empty(&props.geom[..], props.superbloc, "<geom_type> <geom parameters>]");
    fail_if_none(&props.material, props.superbloc, "material <material_name>");


    let special;
    let material: Arc<Box<Material + 'static + Send + Sync>>;
    let transform;
    let normals;
    let solid;
    let flat;
    let refl_m;
    let refl_a;
    let alpha: f32;
    let refr_c;

    {
        solid     = props.solid;
        flat      = props.flat;
        let mname = &props.material.as_ref().unwrap().1;
        special   = &mname[..] == "uvs" || &mname[..] == "normals";

        match mtllib.get(mname) {
            None => panic!("Attempted to use an unknown material: {}", *mname),
            Some(&(ref a, ref m)) => {
                alpha    = a.clone();
                material = m.clone();
            }
        }

        let pos       = props.pos.as_ref().unwrap().1;
        let mut angle = props.angle.as_ref().unwrap().1;

        angle.x = angle.x.to_radians();
        angle.y = angle.y.to_radians();
        angle.z = angle.z.to_radians();

        transform = Isometry3::new(pos.coords, angle);
        normals   = None;

        let refl_param = props.refl.unwrap_or((props.superbloc, Vector2::new(0.0, 0.0))).1;
        refl_m = refl_param.x as f32;
        refl_a = refl_param.y as f32;

        let refr_param = props.refr.unwrap_or((props.superbloc, 1.0)).1;
        refr_c = refr_param as f64;
    }

    if props.geom.len() > 1 {
        let mut geoms = Vec::new();

        for &(ref l, ref g) in props.geom.iter() {
            match *g {
                Shape::GBall(ref r) => geoms.push(Box::new(Ball::new(*r)) as Box<SupportMapShape + Send + Sync>),
                Shape::GCuboid(ref rs) => geoms.push(Box::new(Cuboid::new(*rs)) as Box<SupportMapShape + Send + Sync>),
                Shape::GCylinder(ref h, ref r) => geoms.push(Box::new(Cylinder::new(*h, *r)) as Box<SupportMapShape + Send + Sync>),
                Shape::GCapsule(ref h, ref r) => geoms.push(Box::new(Capsule::new(*h, *r)) as Box<SupportMapShape + Send + Sync>),
                Shape::GCone(ref h, ref r) => geoms.push(Box::new(Cone::new(*h, *r)) as Box<SupportMapShape + Send + Sync>),
                _ => println!("Warning: unsuported geometry on a minkosky sum at line {}.", *l)
            }
        }

        if !geoms.is_empty() {
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, Box::new(MinkowksiSum::new(geoms)), normals, solid)));
            return;
        }
    }

    match props.geom[0].1.clone() {
        Shape::GBall(r) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, Box::new(Ball::new(r)), normals, solid))),
        Shape::GCuboid(rs) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, Box::new(Cuboid::new(rs)), normals, solid))),
        Shape::GCylinder(h, r) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, Box::new(Cylinder::new(h, r)), normals, solid))),
        Shape::GCapsule(h, r) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, Box::new(Capsule::new(h, r)), normals, solid))),
        Shape::GCone(h, r) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, Box::new(Cone::new(h, r)), normals, solid))),
        Shape::GPlane(n) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, Box::new(Plane::new(n)), normals, solid))),
        Shape::GObj(objpath, mtlpath) => {
            let mtlpath = Path::new(&mtlpath[..]);
            let os      = obj::parse_file(&Path::new(&objpath[..]), &mtlpath, "").unwrap();

            if os.len() > 0 {
                let coords: Arc<Vec<Point3<f64>>> = Arc::new(os[0].1.coords().iter().map(|a| Point3::new(a.x as f64, a.y as f64, a.z as f64) / 4.0f64).collect()); // XXX: remove this arbitrary division by 4.0!
                let uvs: Arc<Vec<Point2<f64>>>    = Arc::new(os[0].1.uvs().iter().flat_map(|a| vec!(Point2::new(a.x as f64, a.y as f64)).into_iter()).collect());
                let ns: Arc<Vec<Vector3<f64>>> = Arc::new(os[0].1.normals().iter().map(|a| Vector3::new(a.x as f64, a.y as f64, a.z as f64)).collect());

                for n in ns.iter() {
                    if *n != *n {
                        panic!("This normal is wrong: {:?}", *n);
                    }
                }

                for (_, o, mat) in os.into_iter() {
                    let mut o = o;
                    let faces = Arc::new(o.mut_faces().unwrap());

                    let mesh: Box<TriMesh<Point3<f64>>>;
                    
                    if flat {
                        mesh = Box::new(TriMesh::new(coords.clone(), faces, Some(uvs.clone()), None));
                    }
                    else {
                        mesh = Box::new(TriMesh::new(coords.clone(), faces, Some(uvs.clone()), Some(ns.clone())));
                    }
                    match mat {
                        Some(m) => {
                            let t = match m.diffuse_texture {
                                None        => None,
                                Some(ref t) => {
                                    let mut p = PathBuf::new();
                                    p.push(mtlpath);
                                    p.push(&t[..]);

                                    {
                                        let file = File::open(p.clone());

                                        if file.is_err() {
                                            panic!(format!("Image not found: {}", p.to_str().unwrap()));
                                        }
                                    }

                                    Texture2d::from_png(&p, false, Interpolation::Bilinear, Overflow::Wrap)
                                }
                            };

                            let a = match m.opacity_map {
                                None        => None,
                                Some(ref a) => {
                                    let mut p = PathBuf::new();
                                    p.push(mtlpath);
                                    p.push(&a[..]);

                                    {
                                        let file = File::open(p.clone());

                                        if file.is_err() {
                                            panic!(format!("Image not found: {}", p.to_str().unwrap()));
                                        }
                                    }

                                    Texture2d::from_png(&p, true, Interpolation::Bilinear, Overflow::Wrap)
                                }
                            };

                            let alpha = m.alpha * alpha;
                            let color = Arc::new(Box::new(PhongMaterial::new(
                                m.ambiant,
                                m.diffuse,
                                m.specular,
                                t,
                                a,
                                m.shininess
                                )) as Box<Material + 'static + Send + Sync>);

                            nodes.push(Arc::new(SceneNode::new(if special { material.clone() } else { color }, refl_m, refl_a, alpha, refr_c, transform, mesh, None, solid)));
                        },
                        None => nodes.push(Arc::new(SceneNode::new(material.clone(), refl_m, refl_a, alpha, refr_c, transform, mesh, None, solid)))
                    }
                }
            }
        }
    }
}

fn parse_triplet<'a>(l: usize, mut ws: SplitWhitespace<'a>) -> Vector3<f64> {
    let sx = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 0."));
    let sy = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 1."));
    let sz = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 2."));

    let x: Result<f64, _> = FromStr::from_str(sx);
    let y: Result<f64, _> = FromStr::from_str(sy);
    let z: Result<f64, _> = FromStr::from_str(sz);

    let x = x.unwrap_or_else(|_| error(l, &format!("failed to parse `{}' as a f64.", sx)[..]));
    let y = y.unwrap_or_else(|_| error(l, &format!("failed to parse `{}' as a f64.", sy)[..]));
    let z = z.unwrap_or_else(|_| error(l, &format!("failed to parse `{}' as a f64.", sz)[..]));

    Vector3::new(x, y, z)
}

fn parse_name<'a>(_: usize, ws: SplitWhitespace<'a>) -> String {
    let res: Vec<&'a str> = ws.collect();
    res.join(" ")
}

fn parse_number<'a>(l: usize, mut ws: SplitWhitespace<'a>) -> f64 {
    let sx = ws.next().unwrap_or_else(|| error(l, "1 component was expected, found 0."));

    let x: Result<f64, _> = FromStr::from_str(sx);

    let x = x.unwrap_or_else(|_| error(l, &format!("failed to parse `{}' as a f64.", sx)[..]));

    x
}

fn parse_duet<'a>(l: usize, mut ws: SplitWhitespace<'a>) -> Vector2<f64> {
    let sx = ws.next().unwrap_or_else(|| error(l, "2 components were expected, found 0."));
    let sy = ws.next().unwrap_or_else(|| error(l, "2 components were expected, found 1."));

    let x: Result<f64, _> = FromStr::from_str(sx);
    let y: Result<f64, _> = FromStr::from_str(sy);

    let x = x.unwrap_or_else(|_| error(l, &format!("failed to parse `{}' as a f64.", sx)[..]));
    let y = y.unwrap_or_else(|_| error(l, &format!("failed to parse `{}' as a f64.", sy)[..]));

    Vector2::new(x, y)
}

fn parse_ball<'a>(l: usize, ws: SplitWhitespace<'a>) -> Shape {
    let radius = parse_number(l, ws);

    Shape::GBall(radius)
}

fn parse_box<'a>(l: usize, ws: SplitWhitespace<'a>) -> Shape {
    let extents = parse_triplet(l, ws);

    Shape::GCuboid(extents)
}

fn parse_plane<'a>(l: usize, ws: SplitWhitespace<'a>) -> Shape {
    let normal = na::normalize(&parse_triplet(l, ws));

    Shape::GPlane(normal)
}

fn parse_cylinder<'a>(l: usize, ws: SplitWhitespace<'a>) -> Shape {
    let v = parse_duet(l, ws);

    Shape::GCylinder(v.x, v.y)
}

fn parse_capsule<'a>(l: usize, ws: SplitWhitespace<'a>) -> Shape {
    let v = parse_duet(l, ws);

    Shape::GCapsule(v.x, v.y)
}

fn parse_cone<'a>(l: usize, ws: SplitWhitespace<'a>) -> Shape {
    let v = parse_duet(l, ws);

    Shape::GCone(v.x, v.y)
}

fn parse_obj<'a>(l: usize, mut ws: SplitWhitespace<'a>) -> Shape {
    let objpath = ws.next().unwrap_or_else(|| error(l, "2 paths were expected, found 0."));
    let mtlpath = ws.next().unwrap_or_else(|| error(l, "2 paths were expected, found 1."));

    Shape::GObj(objpath.to_string(), mtlpath.to_string())
}

trait SupportMapShape : SupportMap<Point, Matrix> + 'static + Send + Sync +
                        RayCast<Point, Matrix> + HasBoundingVolume<Matrix, AABB<Point>> { }

impl<T> SupportMapShape for T
    where T: SupportMap<Point, Matrix> + 'static + Send + Sync + RayCast<Point, Matrix> +
             HasBoundingVolume<Matrix, AABB<Point>>
{ }

struct MinkowksiSum {
    geoms: Vec<Box<SupportMapShape + Sync + Send>>
}

impl MinkowksiSum {
    pub fn new(geoms: Vec<Box<SupportMapShape + Sync + Send>>) -> MinkowksiSum {
        MinkowksiSum {
            geoms: geoms
        }
    }
}

impl HasBoundingVolume<Matrix, AABB<Point>> for MinkowksiSum {
    fn bounding_volume(&self, m: &Matrix) -> AABB<Point> {
        support_map_aabb(m, self)
    }
}

impl SupportMap<Point, Matrix> for MinkowksiSum {
    fn support_point(&self, transform: &Matrix, dir: &Vect) -> Point {
        let mut pt  = na::origin::<Point>();
        let new_dir = transform.inverse_transform_vector(dir);

        for i in self.geoms.iter() {
            pt = pt + i.support_point(&na::one(), &new_dir).coords
        }

        transform * pt
    }
}

impl RayCast<Point, Matrix> for MinkowksiSum {
    fn toi_and_normal_with_ray(&self, m: &Matrix, ray: &Ray3<f64>, solid: bool) -> Option<RayIntersection3<f64>> {
        ray_internal::implicit_toi_and_normal_with_ray(
            m, self, &mut JohnsonSimplex::<Point>::new_w_tls(), ray, solid)
    }
}
