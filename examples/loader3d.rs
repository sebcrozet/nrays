#![warn(non_camel_case_types)]
#![feature(globs)]

extern crate native;
extern crate green;
extern crate png;
extern crate "nalgebra" as na;
extern crate "ncollide3df64" as ncollide;
extern crate "nrays3d"       as nrays;

use std::from_str::from_str;
use std::io::fs::{File, PathExtensions};
use std::os;
use std::mem;
use std::str;
use std::str::Words;
use std::collections::HashMap;
use std::sync::Arc;
use na::{Pnt2, Pnt3, Vec2, Vec3, Iso3};
use ncollide::bounding_volume::{AABB, HasAABB, implicit_shape_aabb};
use ncollide::geom::{Plane, Ball, Cone, Cylinder, Capsule, Cuboid, Mesh};
use ncollide::implicit::Implicit;
use ncollide::ray::{Ray, RayCast, RayIntersection, implicit_toi_and_normal_with_ray};
use ncollide::math::{Point, Vect, Matrix};
use ncollide::narrow::algorithm::johnson_simplex::JohnsonSimplex;
use ncollide::geom::Geom;
use nrays::scene_node::SceneNode;
use nrays::material::Material;
use nrays::normal_material::NormalMaterial;
use nrays::phong_material::PhongMaterial;
use nrays::texture2d::{Texture2d, Bilinear, Wrap};
use nrays::uv_material::UVMaterial;
use nrays::scene::Scene;
use nrays::scene;
use nrays::light::Light;
use nrays::obj;
use nrays::mtl;

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, proc() {
        main();
    })
}

fn main() {
    let args = os::args();

    if args.len() != 2 {
        fail!("Usage: {} scene_file", *args.get(0));
    }

    let path = Path::new(args[1].clone());

    if !path.exists() {
        fail!("Unable to find the file: {}", path.as_str().unwrap())
    }

    println!("Loading the scene.");
    let s     = File::open(&path).unwrap().read_to_end().unwrap(); // FIXME: display an error message?
    let descr = str::from_utf8(s.as_slice()).unwrap();

    let (lights, nodes, cameras) = parse(descr.as_slice());
    let nnodes    = nodes.len();
    let nlights   = lights.len();
    let ncams     = cameras.len();
    let scene = Arc::new(Scene::new(nodes, lights, na::one()));
    println!("Scene loaded. {} lights, {} objects, {} cameras.", nlights, nnodes, ncams);

    for c in cameras.into_iter() {
        // FIXME: new_perspective is _not_ accessible as a free function.
        let perspective = na::perspective3d(
            c.resolution.x,
            c.resolution.y,
            c.fovy.to_radians(),
            1.0,
            100000.0);

        let mut camera = na::one::<Iso3<f64>>();
        camera.look_at_z(&c.eye, &c.at, &Vec3::y());

        let projection = na::to_homogeneous(&camera) * na::inv(&perspective).expect("Perspective matrix not invesible.");

        println!("Casting {} rays per pixels (win. {}).", c.aa.x as uint, c.aa.y);

        let pixels = scene::render(&scene, &c.resolution, c.aa.x as uint, c.aa.y, c.eye, projection);

        println!("Rays cast.");

        println!("Saving image to: {:s}", c.output);
        pixels.to_png(&Path::new(c.output));
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
    GeomMode,
    CameraMode,
    NoMode
}

#[deriving(Clone)]
enum Geometry {
    GBall(f64),
    GPlane(Vec3<f64>),
    GCuboid(Vec3<f64>),
    GCylinder(f64, f64),
    GCapsule(f64, f64),
    GCone(f64, f64),
    GObj(String, String),
}

struct Camera {
    eye:        Pnt3<f64>,
    at:         Pnt3<f64>,
    fovy:       f64,
    resolution: Vec2<f64>,
    aa:         Vec2<f64>,
    output:     String
}

impl Camera {
    pub fn new(eye: Pnt3<f64>, at: Pnt3<f64>, fovy: f64, resolution: Vec2<f64>, aa: Vec2<f64>, output: String) -> Camera {
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
    superbloc:  uint,
    geom:       Vec<(uint, Geometry)>,
    pos:        Option<(uint, Pnt3<f64>)>,
    angle:      Option<(uint, Vec3<f64>)>,
    material:   Option<(uint, String)>,
    eye:        Option<(uint, Pnt3<f64>)>,
    at:         Option<(uint, Pnt3<f64>)>,
    fovy:       Option<(uint, f64)>,
    color:      Option<(uint, Pnt3<f64>)>,
    resolution: Option<(uint, Vec2<f64>)>,
    output:     Option<(uint, String)>,
    refl:       Option<(uint, Vec2<f64>)>,
    refr:       Option<(uint, f64)>,
    aa:         Option<(uint, Vec2<f64>)>,
    radius:     Option<(uint, f64)>,
    nsample:    Option<(uint, f64)>,
    solid:      bool,
    flat:       bool,
}

impl Properties {
    pub fn new(l: uint) -> Properties {
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

fn error(line: uint, err: &str) -> ! {
    fail!("At line {}: {}", line, err)
}

fn warn(line: uint, err: &str) {
    println!("At line {}: {}", line, err)
}

fn parse(string: &str) -> (Vec<Light>, Vec<Arc<SceneNode>>, Vec<Camera>) {
    let mut nodes   = Vec::new();
    let mut lights  = Vec::new();
    let mut cameras = Vec::new();
    let mut props   = Properties::new(0);
    let mut mode    = NoMode;
    let mut mtllib  = HashMap::new();

    for (l, line) in string.lines_any().enumerate() {
        let mut words  = line.words();
        let tag        = words.next();

        let white = Arc::new(box PhongMaterial::new(
            Pnt3::new(0.1, 0.1, 0.1),
            Pnt3::new(1.0, 1.0, 1.0),
            Pnt3::new(1.0, 1.0, 1.0),
            None,
            None,
            100.0
        ) as Box<Material + Send + Sync>);

        mtllib.insert("normals".to_string(), (1.0, Arc::new(box NormalMaterial::new() as Box<Material + Send + Sync>)));
        mtllib.insert("uvs".to_string(), (1.0, Arc::new(box UVMaterial::new() as Box<Material + Send + Sync>)));
        mtllib.insert("default".to_string(), (1.0, white));

        match tag {
            None    => { },
            Some(w) => {
                if w.len() != 0 && w.as_bytes()[0] != ('#' as u8) {
                    match w {
                        // top-level commands
                        "mtllib"   => register_mtllib(parse_name(l, words).as_slice(), &mut mtllib),
                        "light"    => {
                            let old = mem::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = LightMode;
                        },
                        "geometry" => {
                            let old = mem::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = GeomMode;
                        },
                        "camera"   => {
                            let old = mem::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = CameraMode;
                        },
                        // common attributes
                        "color"      => props.color      = Some((l, parse_triplet(l, words).to_pnt())),
                        "angle"      => props.angle      = Some((l, parse_triplet(l, words))),
                        "pos"        => props.pos        = Some((l, parse_triplet(l, words).to_pnt())),
                        "eye"        => props.eye        = Some((l, parse_triplet(l, words).to_pnt())),
                        "at"         => props.at         = Some((l, parse_triplet(l, words).to_pnt())),
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
                            println!("Warning: unknown line {} ignored: `{:s}'", l, line);
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
            mtllib:  &mut HashMap<String, (f32, Arc<Box<Material + Send + Sync>>)>,
            lights:  &mut Vec<Light>,
            nodes:   &mut Vec<Arc<SceneNode>>,
            cameras: &mut Vec<Camera>) {
    match *mode {
        LightMode  => register_light(props, lights),
        GeomMode   => register_geometry(props, mtllib, nodes),
        CameraMode => register_camera(props, cameras),
        NoMode     => register_nothing(props),
    }
}

fn warn_if_not_empty<T>(t: &[(uint, T)]) {
    if !t.is_empty() {
        for g in t.iter() {
            warn(g.ref0().clone(), "dropped unexpected attribute.")
        }
    }
}

fn warn_if_some<T>(t: &Option<(uint, T)>) {
    t.as_ref().map(|&(l, _)| warn(l, "dropped unexpected attribute."));
}

fn fail_if_empty<T>(t: &[(uint, T)], l: uint, attrname: &str) {
    if t.is_empty() {
        error(l, format!("missing attribute: {}", attrname).as_slice());
    }
}

fn fail_if_none<T>(t: &Option<(uint, T)>, l: uint, attrname: &str) {
    match *t {
        None    => error(l, format!("missing attribute: {}", attrname).as_slice()),
        Some(_) => { }
    }
}

fn register_nothing(props: Properties) {
    warn_if_not_empty(props.geom.as_slice());
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
    warn_if_not_empty(props.geom.as_slice());
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


    let aa   = props.aa.unwrap_or((l, Vec2::new(1.0, 0.0)));
    let eye  = props.eye.unwrap().val1();
    let at   = props.at.unwrap().val1();
    let fov  = props.fovy.unwrap().val1();
    let res  = props.resolution.unwrap().val1();
    let name = props.output.unwrap().val1();

    cameras.push(Camera::new(eye, at, fov, res, aa.val1(), name));
}

fn register_light(props: Properties, lights: &mut Vec<Light>) {
    warn_if_not_empty(props.geom.as_slice());
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

    let radius  = props.radius.unwrap_or((props.superbloc, 0.0)).val1();
    let nsample = props.nsample.unwrap_or((props.superbloc, 1.0)).val1();
    let pos     = props.pos.unwrap().val1();
    let color: Pnt3<f32> = na::cast(props.color.unwrap().val1());
    let light   = Light::new(pos, radius, nsample as uint, color);

    lights.push(light);
}

fn register_mtllib(path: &str, mtllib: &mut HashMap<String, (f32, Arc<Box<Material + Send + Sync>>)>) {
    let materials = mtl::parse_file(&Path::new(path)).unwrap(); // expect(format!("Failed to parse the mtl file: {}", path));

    for m in materials.into_iter() {
        let t = m.diffuse_texture.as_ref().map(|t| Texture2d::from_png(&Path::new(t.as_slice()), false, Bilinear, Wrap).expect("Image not found."));

        let a = m.opacity_map.as_ref().map(|t| Texture2d::from_png(&Path::new(t.as_slice()), true, Bilinear, Wrap).expect("Image not found."));

        let alpha = m.alpha;

        let color = box PhongMaterial::new(
            m.ambiant,
            m.diffuse,
            m.specular,
            t,
            a,
            m.shininess
            ) as Box<Material + Send + Sync>;

        mtllib.insert(m.name, (alpha, Arc::new(color)));
    }
}

fn register_geometry(props:  Properties,
                     mtllib: &mut HashMap<String, (f32, Arc<Box<Material + Send + Sync>>)>,
                     nodes:  &mut Vec<Arc<SceneNode>>) {
    warn_if_some(&props.eye);
    warn_if_some(&props.at);
    warn_if_some(&props.fovy);
    warn_if_some(&props.color);
    warn_if_some(&props.output);
    warn_if_some(&props.resolution);

    fail_if_none(&props.pos, props.superbloc, "pos <x> <y> <z>");
    fail_if_none(&props.angle, props.superbloc, "color <r> <g> <b>");
    fail_if_empty(props.geom.as_slice(), props.superbloc, "<geom_type> <geom parameters>]");
    fail_if_none(&props.material, props.superbloc, "material <material_name>");


    let special;
    let material;
    let transform;
    let normals;
    let solid;
    let flat;
    let refl_m;
    let refl_a;
    let alpha;
    let refr_c;

    {
        solid     = props.solid;
        flat      = props.flat;
        let mname = props.material.as_ref().unwrap().ref1();
        special   = mname.as_slice() == "uvs" || mname.as_slice() == "normals";
        let (a, m)= mtllib.find(mname).unwrap_or_else(|| fail!("Attempted to use an unknown material: {}", *mname)).clone();

        alpha    = a;
        material = m;

        let pos       = props.pos.as_ref().unwrap().val1();
        let mut angle = props.angle.as_ref().unwrap().val1();

        angle.x = angle.x.to_radians();
        angle.y = angle.y.to_radians();
        angle.z = angle.z.to_radians();

        transform = Iso3::new(pos.to_vec(), angle);
        normals   = None;

        let refl_param = props.refl.unwrap_or((props.superbloc, Vec2::new(0.0, 0.0))).val1();
        refl_m = refl_param.x as f32;
        refl_a = refl_param.y as f32;

        let refr_param = props.refr.unwrap_or((props.superbloc, 1.0)).val1();
        refr_c = refr_param as f64;
    }

    if props.geom.len() > 1 {
        let mut geoms = Vec::new();

        for &(ref l, ref g) in props.geom.iter() {
            match *g {
                GBall(ref r) => geoms.push(box Ball::new(*r) as Box<ImplicitGeom + Send + Sync>),
                GCuboid(ref rs) => geoms.push(box Cuboid::new(*rs) as Box<ImplicitGeom + Send + Sync>),
                GCylinder(ref h, ref r) => geoms.push(box Cylinder::new(*h, *r) as Box<ImplicitGeom + Send + Sync>),
                GCapsule(ref h, ref r) => geoms.push(box Capsule::new(*h, *r) as Box<ImplicitGeom + Send + Sync>),
                GCone(ref h, ref r) => geoms.push(box Cone::new(*h, *r) as Box<ImplicitGeom + Send + Sync>),
                _ => println!("Warning: unsuported geometry on a minkosky sum at line {}.", *l)
            }
        }

        if !geoms.is_empty() {
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, box MinkowksiSum::new(geoms), normals, solid)));
            return;
        }
    }

    match props.geom[0].ref1().clone() {
        GBall(r) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, box Ball::new(r), normals, solid))),
        GCuboid(rs) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, box Cuboid::new(rs), normals, solid))),
        GCylinder(h, r) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, box Cylinder::new(h, r), normals, solid))),
        GCapsule(h, r) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, box Capsule::new(h, r), normals, solid))),
        GCone(h, r) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, box Cone::new(h, r), normals, solid))),
        GPlane(n) =>
            nodes.push(Arc::new(SceneNode::new(material, refl_m, refl_a, alpha, refr_c, transform, box Plane::new(n), normals, solid))),
        GObj(objpath, mtlpath) => {
            let mtlpath = Path::new(mtlpath);
            let os      = obj::parse_file(&Path::new(objpath), &mtlpath, "").unwrap();

            if os.len() > 0 {
                let coords: Arc<Vec<Pnt3<f64>>> = Arc::new(os[0].ref1().coords().iter().map(|a| Pnt3::new(a.x as f64, a.y as f64, a.z as f64) / 4.0f64).collect()); // XXX: remove this arbitrary division by 4.0!
                let uvs: Arc<Vec<Pnt2<f64>>>    = Arc::new(os[0].ref1().uvs().iter().flat_map(|a| vec!(Pnt2::new(a.x as f64, a.y as f64)).into_iter()).collect());
                let ns: Arc<Vec<Vec3<f64>>> = Arc::new(os[0].ref1().normals().iter().map(|a| Vec3::new(a.x as f64, a.y as f64, a.z as f64)).collect());

                for n in ns.iter() {
                    if *n != *n {
                        fail!("This normal is wrong: {}", *n);
                    }
                }

                for (_, o, mat) in os.into_iter() {
                    let mut o = o;
                    let faces = o.mut_faces().unwrap();
                    let faces = Arc::new(faces.iter().flat_map(|a| vec!(a.x as uint, a.y as uint, a.z as uint).into_iter()).collect());

                    let mesh;
                    
                    if flat {
                        mesh = box Mesh::new(coords.clone(), faces, Some(uvs.clone()), None);
                    }
                    else {
                        mesh = box Mesh::new(coords.clone(), faces, Some(uvs.clone()), Some(ns.clone()));
                    }
                    match mat {
                        Some(m) => {
                            let t = match m.diffuse_texture {
                                None        => None,
                                Some(ref t) => {
                                    let mut p = mtlpath.clone();
                                    p.push(t.as_slice());

                                    if !p.exists() {
                                        fail!(format!("Image not found: {}", p.as_str()));
                                    }

                                    Texture2d::from_png(&p, false, Bilinear, Wrap)
                                }
                            };

                            let a = match m.opacity_map {
                                None        => None,
                                Some(ref a) => {
                                    let mut p = mtlpath.clone();
                                    p.push(a.as_slice());

                                    if !p.exists() {
                                        fail!(format!("Image not found: {}", p.as_str()));
                                    }

                                    Texture2d::from_png(&p, true, Bilinear, Wrap)
                                }
                            };

                            let alpha = m.alpha * alpha;
                            let color = Arc::new(box PhongMaterial::new(
                                m.ambiant,
                                m.diffuse,
                                m.specular,
                                t,
                                a,
                                m.shininess
                                ) as Box<Material + Send + Sync>);

                            nodes.push(Arc::new(SceneNode::new(if special { material.clone() } else { color }, refl_m, refl_a, alpha, refr_c, transform, mesh, None, solid)));
                        },
                        None => nodes.push(Arc::new(SceneNode::new(material.clone(), refl_m, refl_a, alpha, refr_c, transform, mesh, None, solid)))
                    }
                }
            }
        }
    }
}

fn parse_triplet<'a>(l: uint, mut ws: Words<'a>) -> Vec3<f64> {
    let sx = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 0."));
    let sy = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 1."));
    let sz = ws.next().unwrap_or_else(|| error(l, "3 components were expected, found 2."));

    let x: Option<f64> = from_str(sx);
    let y: Option<f64> = from_str(sy);
    let z: Option<f64> = from_str(sz);

    let x = x.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f64.", sx).as_slice()));
    let y = y.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f64.", sy).as_slice()));
    let z = z.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f64.", sz).as_slice()));

    Vec3::new(x, y, z)
}

fn parse_name<'a>(_: uint, mut ws: Words<'a>) -> String {
    let res: Vec<&'a str> = ws.collect();
    res.connect(" ")
}

fn parse_number<'a>(l: uint, mut ws: Words<'a>) -> f64 {
    let sx = ws.next().unwrap_or_else(|| error(l, "1 component was expected, found 0."));

    let x: Option<f64> = from_str(sx);

    let x = x.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f64.", sx).as_slice()));

    x
}

fn parse_duet<'a>(l: uint, mut ws: Words<'a>) -> Vec2<f64> {
    let sx = ws.next().unwrap_or_else(|| error(l, "2 components were expected, found 0."));
    let sy = ws.next().unwrap_or_else(|| error(l, "2 components were expected, found 1."));

    let x: Option<f64> = from_str(sx);
    let y: Option<f64> = from_str(sy);

    let x = x.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f64.", sx).as_slice()));
    let y = y.unwrap_or_else(|| error(l, format!("failed to parse `{}' as a f64.", sy).as_slice()));

    Vec2::new(x, y)
}

fn parse_ball<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let radius = parse_number(l, ws);

    GBall(radius)
}

fn parse_box<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let extents = parse_triplet(l, ws);

    GCuboid(extents)
}

fn parse_plane<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let normal = na::normalize(&parse_triplet(l, ws));

    GPlane(normal)
}

fn parse_cylinder<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let v = parse_duet(l, ws);

    GCylinder(v.x, v.y)
}

fn parse_capsule<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let v = parse_duet(l, ws);

    GCapsule(v.x, v.y)
}

fn parse_cone<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let v = parse_duet(l, ws);

    GCone(v.x, v.y)
}

fn parse_obj<'a>(l: uint, mut ws: Words<'a>) -> Geometry {
    let objpath = ws.next().unwrap_or_else(|| error(l, "2 paths were expected, found 0."));
    let mtlpath = ws.next().unwrap_or_else(|| error(l, "2 paths were expected, found 1."));

    GObj(objpath.to_string(), mtlpath.to_string())
}

trait ImplicitGeom : Implicit<Point, Vect, Matrix> + Geom { }

impl<T: Implicit<Point, Vect, Matrix> + Geom> ImplicitGeom for T { }

struct MinkowksiSum {
    geoms: Vec<Box<ImplicitGeom + Sync + Send>>
}

impl MinkowksiSum {
    pub fn new(geoms: Vec<Box<ImplicitGeom + Sync + Send>>) -> MinkowksiSum {
        MinkowksiSum {
            geoms: geoms
        }
    }
}

impl HasAABB for MinkowksiSum {
    fn aabb(&self, m: &Matrix) -> AABB {
        implicit_shape_aabb(m, self)
    }
}

impl Implicit<Point, Vect, Matrix> for MinkowksiSum {
    fn support_point(&self, transform: &Matrix, dir: &Vect) -> Point {
        let mut pt  = na::orig::<Point>();
        let new_dir = na::inv_rotate(transform, dir);

        for i in self.geoms.iter() {
            pt = pt + i.support_point(&na::one(), &new_dir).to_vec()
        }

        na::transform(transform, &pt)
    }
}

impl RayCast for MinkowksiSum {
    fn toi_and_normal_with_ray(&self, ray: &Ray, solid: bool) -> Option<RayIntersection> {
        implicit_toi_and_normal_with_ray(&na::one(), self, &mut JohnsonSimplex::<Point, Vect>::new_w_tls(), ray, solid)
    }
}
