#[crate_id = "loader_3d"];
#[crate_type = "bin"];
#[warn(non_camel_case_types)];
#[feature(managed_boxes)];
#[feature(globs)];

extern mod png;
extern mod extra;
extern mod nalgebra;
extern mod ncollide = "ncollide3df64";
extern mod nrays    = "nrays3d";

use std::io::fs::File;
use std::os;
use std::util;
use std::str;
use std::str::Words;
use std::hashmap::HashMap;
use extra::arc::Arc;
use nalgebra::na::{Vec2, Vec3, Vec4, Iso3};
use nalgebra::na;
use ncollide::ray::Ray;
use ncollide::geom::{Plane, Ball, Cone, Cylinder, Box, Mesh};
use nrays::scene_node::SceneNode;
use nrays::material::Material;
use nrays::normal_material::NormalMaterial;
use nrays::phong_material::PhongMaterial;
use nrays::texture2d::{Texture2d, Bilinear, Wrap};
use nrays::reflective_material::ReflectiveMaterial;
use nrays::uv_material::UVMaterial;
use nrays::scene::Scene;
use nrays::light::Light;
use nrays::obj;
use nrays::mtl;

fn main() {
    let args = os::args();

    if args.len() != 2 {
        fail!("Usage: " + args[0] + " scene_file");
    }

    let path = Path::new(args[1]);

    if !path.exists() {
        fail!("Unable to find the file: " + path.as_str().unwrap())
    }

    println!("Loading the scene.");
    let s     = File::open(&path).expect("Cannot open the file: " + path.as_str().unwrap()).read_to_end();
    let descr = str::from_utf8_owned(s);

    let (lights, nodes, cameras) = parse(descr);
    let nnodes  = nodes.len();
    let nlights = lights.len();
    let ncams   = cameras.len();
    let scene   = Scene::new(nodes, lights);
    println!("Scene loaded. {} lights, {} objects, {} cameras.", nlights, nnodes, ncams);

    for c in cameras.move_iter() {
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
        let pixels = scene.render(&c.resolution, c.aa.x as uint, c.aa.y, |pt| {
            let device_x = (pt.x / c.resolution.x - 0.5) * 2.0;
            let device_y = -(pt.y / c.resolution.y - 0.5) * 2.0;
            let start = Vec4::new(device_x, device_y, -1.0, 1.0);
            let h_eye = projection * start;
            let eye: Vec3<f64> = na::from_homogeneous(&h_eye);
            Ray::new(c.eye, na::normalize(&(eye - c.eye)))
        });
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
    Light,
    Geometry,
    Camera,
    NoMode
}

enum Geometry {
    Ball(f64),
    Plane(Vec3<f64>),
    Box(Vec3<f64>),
    Cylinder(f64, f64),
    Cone(f64, f64),
    Obj(~str, ~str),
}

struct Camera {
    eye:        Vec3<f64>,
    at:         Vec3<f64>,
    fovy:       f64,
    resolution: Vec2<f64>,
    aa:         Vec2<f64>,
    output:     ~str
}

impl Camera {
    pub fn new(eye: Vec3<f64>, at: Vec3<f64>, fovy: f64, resolution: Vec2<f64>, aa: Vec2<f64>, output: ~str) -> Camera {
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
    superbloc: uint,
    geom:       Option<(uint, Geometry)>,
    pos:        Option<(uint, Vec3<f64>)>,
    angle:      Option<(uint, Vec3<f64>)>,
    material:   Option<(uint, ~str)>,
    eye:        Option<(uint, Vec3<f64>)>,
    at:         Option<(uint, Vec3<f64>)>,
    fovy:       Option<(uint, f64)>,
    color:      Option<(uint, Vec3<f64>)>,
    resolution: Option<(uint, Vec2<f64>)>,
    output:     Option<(uint, ~str)>,
    refl:       Option<(uint, Vec2<f64>)>,
    aa:         Option<(uint, Vec2<f64>)>,
    radius:     Option<(uint, f64)>,
    nsample:    Option<(uint, f64)>,
}

impl Properties {
    pub fn new(l: uint) -> Properties {
        Properties {
            superbloc:  l,
            geom:       None,
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
            aa:         None,
            radius:     None,
            nsample:    None,
        }
    }
}

fn error(line: uint, err: &str) -> ! {
    fail!("At line " + line.to_str() + ": " + err)
}

fn warn(line: uint, err: &str) {
    println!("At line {}: {}", line, err)
}

fn parse(string: &str) -> (~[Light], ~[@SceneNode], ~[Camera]) {
    let mut nodes   = ~[];
    let mut lights  = ~[];
    let mut cameras = ~[];
    let mut props   = Properties::new(0);
    let mut mode    = NoMode;
    let mut mtllib  = HashMap::new();

    for (l, line) in string.lines_any().enumerate() {
        let mut words  = line.words();
        let tag        = words.next();

        let white = @PhongMaterial::new(
            Vec3::new(0.1, 0.1, 0.1),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            None,
            100.0
        ) as @Material;

        mtllib.insert(~"normals", @NormalMaterial::new() as @Material);
        mtllib.insert(~"uvs", @UVMaterial::new() as @Material);
        mtllib.insert(~"default", white);

        match tag {
            None    => { },
            Some(w) => {
                if w.len() != 0 && w[0] != ('#' as u8) {
                    match w {
                        // top-level commands
                        &"mtllib"   => register_mtllib(parse_name(l, words), &mut mtllib),
                        &"light"    => {
                            let old = util::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = Light;
                        },
                        &"geometry" => {
                            let old = util::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = Geometry;
                        },
                        &"camera"   => {
                            let old = util::replace(&mut props, Properties::new(l));
                            register(&mode, old, &mut mtllib, &mut lights, &mut nodes, &mut cameras);
                            mode  = Camera;
                        },
                        // common attributes
                        &"color"      => props.color      = Some((l, parse_triplet(l, words))),
                        &"angle"      => props.angle      = Some((l, parse_triplet(l, words))),
                        &"pos"        => props.pos        = Some((l, parse_triplet(l, words))),
                        &"eye"        => props.eye        = Some((l, parse_triplet(l, words))),
                        &"at"         => props.at         = Some((l, parse_triplet(l, words))),
                        &"material"   => props.material   = Some((l, parse_name(l, words))),
                        &"fovy"       => props.fovy       = Some((l, parse_number(l, words))),
                        &"output"     => props.output     = Some((l, parse_name(l, words))),
                        &"resolution" => props.resolution = Some((l, parse_duet(l, words))),
                        &"refl"       => props.refl       = Some((l, parse_duet(l, words))),
                        &"aa"         => props.aa         = Some((l, parse_duet(l, words))),
                        &"radius"     => props.radius     = Some((l, parse_number(l, words))),
                        &"nsample"    => props.nsample    = Some((l, parse_number(l, words))),
                        // geometries
                        &"ball"       => props.geom = Some((l, parse_ball(l, words))),
                        &"plane"      => props.geom = Some((l, parse_plane(l, words))),
                        &"box"        => props.geom = Some((l, parse_box(l, words))),
                        &"cylinder"   => props.geom = Some((l, parse_cylinder(l, words))),
                        &"cone"       => props.geom = Some((l, parse_cone(l, words))),
                        &"obj"        => props.geom = Some((l, parse_obj(l, words))),
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
            mtllib:  &mut HashMap<~str, @Material>,
            lights:  &mut ~[Light],
            nodes:   &mut ~[@SceneNode],
            cameras: &mut ~[Camera]) {
    match *mode {
        Light    => register_light(props, lights),
        Geometry => register_geometry(props, mtllib, nodes),
        Camera   => register_camera(props, cameras),
        NoMode   => register_nothing(props),
    }
}

fn warn_if_some<T>(t: &Option<(uint, T)>) {
    t.as_ref().map(|&(l, _)| warn(l, "dropped unexpected attribute."));
}

fn fail_if_none<T>(t: &Option<(uint, T)>, l: uint, attrname: &str) {
    match *t {
        None    => error(l, "missing attribute: " + attrname),
        Some(_) => { }
    }
}

fn register_nothing(props: Properties) {
    warn_if_some(&props.geom);
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
    warn_if_some(&props.aa);
    warn_if_some(&props.radius);
    warn_if_some(&props.nsample);
}

fn register_camera(props: Properties, cameras: &mut ~[Camera]) {
    warn_if_some(&props.geom);
    warn_if_some(&props.pos);
    warn_if_some(&props.angle);
    warn_if_some(&props.material);
    warn_if_some(&props.color);
    warn_if_some(&props.refl);
    warn_if_some(&props.radius);
    warn_if_some(&props.nsample);

    let l = props.superbloc;

    fail_if_none(&props.output, l, "output <filename>");
    fail_if_none(&props.resolution, l, "resolution <x> <y>");
    fail_if_none(&props.eye, l, "eye <x> <y> <z>");
    fail_if_none(&props.at, l, "at <x> <y> <z>");
    fail_if_none(&props.fovy, l, "fovy <value>");


    let aa   = props.aa.unwrap_or((l, Vec2::new(1.0, 0.0)));
    let eye  = props.eye.unwrap().n1();
    let at   = props.at.unwrap().n1();
    let fov  = props.fovy.unwrap().n1();
    let res  = props.resolution.unwrap().n1();
    let name = props.output.unwrap().n1();

    cameras.push(Camera::new(eye, at, fov, res, aa.n1(), name));
}

fn register_light(props: Properties, lights: &mut ~[Light]) {
    warn_if_some(&props.geom);
    warn_if_some(&props.angle);
    warn_if_some(&props.material);
    warn_if_some(&props.eye);
    warn_if_some(&props.at);
    warn_if_some(&props.fovy);
    warn_if_some(&props.output);
    warn_if_some(&props.resolution);
    warn_if_some(&props.refl);
    warn_if_some(&props.aa);

    fail_if_none(&props.pos, props.superbloc, "pos <x> <y> <z>");
    fail_if_none(&props.color, props.superbloc, "color <r> <g> <b>");

    let radius  = props.radius.unwrap_or((props.superbloc, 0.0)).n1();
    let nsample = props.nsample.unwrap_or((props.superbloc, 1.0)).n1();
    let pos     = props.pos.unwrap().n1();
    let color   = na::cast(props.color.unwrap().n1());
    let light   = Light::new(pos, radius, nsample as uint, color);

    lights.push(light);
}

fn register_mtllib(path: &str, mtllib: &mut HashMap<~str, @Material>) {
    let materials = mtl::parse_file(&Path::new(path)).expect("Failed to parse the mtl file: " + path);

    for m in materials.move_iter() {
        let t = m.diffuse_texture.as_ref().map(|t| Texture2d::from_png(&Path::new(t.as_slice()), Bilinear, Wrap).expect("Image not found."));
        let color = @PhongMaterial::new(
            m.ambiant,
            m.diffuse,
            m.specular,
            t,
            m.shininess
            ) as @Material;

        mtllib.insert(m.name, color);
    }
}

fn register_geometry(props:  Properties,
                     mtllib: &mut HashMap<~str, @Material>,
                     nodes:  &mut ~[@SceneNode]) {
    warn_if_some(&props.eye);
    warn_if_some(&props.at);
    warn_if_some(&props.fovy);
    warn_if_some(&props.color);
    warn_if_some(&props.output);
    warn_if_some(&props.resolution);

    fail_if_none(&props.pos, props.superbloc, "pos <x> <y> <z>");
    fail_if_none(&props.angle, props.superbloc, "color <r> <g> <b>");
    fail_if_none(&props.geom, props.superbloc, "<geom_type> <geom parameters>]");
    fail_if_none(&props.material, props.superbloc, "material <material_name>");
    fail_if_none(&props.refl, props.superbloc, "refl <mix> <atenuation>");


    let special;
    let material;
    let refl;
    let transform;
    let normals;

    {
        let mname     = props.material.as_ref().unwrap().n1_ref();
        special   = mname.as_slice() == "uvs" || mname.as_slice() == "normals";
        material  = *mtllib.find(mname).unwrap_or_else(|| fail!("Attempted to use an unknown material: " + *mname));

        let pos       = props.pos.as_ref().unwrap().n1();
        let mut angle = props.angle.as_ref().unwrap().n1();

        angle.x = angle.x.to_radians();
        angle.y = angle.y.to_radians();
        angle.z = angle.z.to_radians();

        let refl_param = props.refl.as_ref().unwrap().n1();
        transform = Iso3::new(pos, angle);
        normals   = None;
        refl = ReflectiveMaterial::new(refl_param.x as f32, refl_param.y as f32);
    }


    match props.geom.unwrap().n1() {
        Ball(r) =>
            nodes.push(@SceneNode::new(material, refl, transform, @Ball::new(r), normals)),
        Box(rs) =>
            nodes.push(@SceneNode::new(material, refl, transform, @Box::new_with_margin(rs, 0.0), normals)),
        Cylinder(h, r) =>
            nodes.push(@SceneNode::new(material, refl, transform, @Cylinder::new_with_margin(h, r, 0.0), normals)),
        Cone(h, r) =>
            nodes.push(@SceneNode::new(material, refl, transform, @Cone::new_with_margin(h, r, 0.0), normals)),
        Plane(n) =>
            nodes.push(@SceneNode::new(material, refl, transform, @Plane::new(n), normals)),
        Obj(objpath, mtlpath) => {
            let mtlpath = Path::new(mtlpath);
            let os      = obj::parse_file(&Path::new(objpath), &mtlpath, "").unwrap();

            if os.len() > 0 {
                let coords = Arc::new(os[0].n1_ref().coords().map(|a| Vec3::new(a.x as f64, a.y as f64, a.z as f64) / 4.0));
                let uvs    = Arc::new(os[0].n1_ref().uvs().flat_map(|a| ~[(a.x as f64, a.y as f64, 0.0)]));
                let ns     = Arc::new(os[0].n1_ref().normals().map(|a| Vec3::new(a.x as f64, a.y as f64, a.z as f64)));

                for n in ns.get().iter() {
                    if *n != *n {
                        fail!("This normal is wrong: {:?}", *n);
                    }
                }

                for (_, o, mat) in os.move_iter() {
                    let mut o = o;
                    let faces = o.mut_faces().unwrap();
                    let faces = Arc::new(faces.flat_map(|a| ~[a.x as uint, a.y as uint, a.z as uint]));

                    let mesh = @Mesh::new_with_margin(coords.clone(), faces, Some(uvs.clone()), Some(ns.clone()), 0.0);
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

                                    Texture2d::from_png(&p, Bilinear, Wrap)
                                }
                            };

                            let color = @PhongMaterial::new(
                                m.ambiant,
                                m.diffuse,
                                m.specular,
                                t,
                                m.shininess
                                ) as @Material;

                            nodes.push(@SceneNode::new(if special { material } else { color }, refl, transform, mesh, None));
                        },
                        None => nodes.push(@SceneNode::new(material, refl, transform, mesh, None))
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

    let x: Option<f64> = FromStr::from_str(sx);
    let y: Option<f64> = FromStr::from_str(sy);
    let z: Option<f64> = FromStr::from_str(sz);

    let x = x.unwrap_or_else(|| error(l, "failed to parse `" + sx + "' as a f64."));
    let y = y.unwrap_or_else(|| error(l, "failed to parse `" + sy + "' as a f64."));
    let z = z.unwrap_or_else(|| error(l, "failed to parse `" + sz + "' as a f64."));

    Vec3::new(x, y, z)
}

fn parse_name<'a>(_: uint, mut ws: Words<'a>) -> ~str {
    ws.to_owned_vec().connect(" ")
}

fn parse_number<'a>(l: uint, mut ws: Words<'a>) -> f64 {
    let sx = ws.next().unwrap_or_else(|| error(l, "1 component was expected, found 0."));

    let x: Option<f64> = FromStr::from_str(sx);

    let x = x.unwrap_or_else(|| error(l, "failed to parse `" + sx + "' as a f64."));

    x
}

fn parse_duet<'a>(l: uint, mut ws: Words<'a>) -> Vec2<f64> {
    let sx = ws.next().unwrap_or_else(|| error(l, "2 components were expected, found 0."));
    let sy = ws.next().unwrap_or_else(|| error(l, "2 components were expected, found 1."));

    let x: Option<f64> = FromStr::from_str(sx);
    let y: Option<f64> = FromStr::from_str(sy);

    let x = x.unwrap_or_else(|| error(l, "failed to parse `" + sx + "' as a f64."));
    let y = y.unwrap_or_else(|| error(l, "failed to parse `" + sy + "' as a f64."));

    Vec2::new(x, y)
}

fn parse_ball<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let radius = parse_number(l, ws);

    Ball(radius)
}

fn parse_box<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let extents = parse_triplet(l, ws);

    Box(extents)
}

fn parse_plane<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let normal = na::normalize(&parse_triplet(l, ws));

    Plane(normal)
}

fn parse_cylinder<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let v = parse_duet(l, ws);

    Cylinder(v.x, v.y)
}

fn parse_cone<'a>(l: uint, ws: Words<'a>) -> Geometry {
    let v = parse_duet(l, ws);

    Cone(v.x, v.y)
}

fn parse_obj<'a>(l: uint, mut ws: Words<'a>) -> Geometry {
    let objpath = ws.next().unwrap_or_else(|| error(l, "2 paths were expected, found 0."));
    let mtlpath = ws.next().unwrap_or_else(|| error(l, "2 paths were expected, found 1."));

    Obj(objpath.to_owned(), mtlpath.to_owned())
}
