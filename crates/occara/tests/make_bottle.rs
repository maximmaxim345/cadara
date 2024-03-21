// use occara::geom::{
//     Axis, CylindericalSurface, Direction, Direction2D, Ellipse2D, Point, Point2D, Transformation,
//     TrimmedCurve2D, Vector,
// };
// use occara::shape::{Compound, Edge, Face, Wire};
// use occara::solid::{Cylinder, Loft, Solid};
// use std::f64::consts::PI;
use occara::geom::{Direction, Point, Transformation, Vector};
use occara::shape::{make_cylinder, Edge, Wire};
use ordered_float::OrderedFloat;

#[allow(unused)]
#[test]
fn test_make_bottle() {
    let width = 50.0;
    let height = 70.0;
    let thickness = 30.0;

    // Define first half of the profile
    let wire = {
        let point1 = Point::new(-width / 2.0, 0.0, 0.0);
        let point2 = Point::new(-width / 2.0, -thickness / 4.0, 0.0);
        let point3 = Point::new(0.0, -thickness / 2.0, 0.0);
        let point4 = Point::new(width / 2.0, -thickness / 4.0, 0.0);
        let point5 = Point::new(width / 2.0, 0.0, 0.0);

        let arc_of_circle = Edge::arc_of_circle(&point2, &point3, &point4);
        let segment1 = Edge::line(&point1, &point2);
        let segment2 = Edge::line(&point4, &point5);

        Wire::new(&[&segment1, &arc_of_circle, &segment2])
    };

    // Mirror the profile to get the full profile
    let mirrored_wire = {
        let axis = Point::origin().axis_with(Direction::x());
        let transformation = Transformation::mirror(axis);
        transformation.apply(wire.clone())
    };

    // Combine the two for the full profile of the bottle
    let bottle_profile = Wire::new(&[&wire, &mirrored_wire]);

    // Extrude the profile to get the body of the bottle
    let body = {
        let face_profile = bottle_profile.make_face();
        let extrude_vec = Vector::new(0.0, 0.0, height);

        face_profile.extrude(&extrude_vec)
    };

    // Chamfer all edges of the bottle
    let mut body = {
        let fillet_radius = thickness / 12.0;
        let mut make_fillet = body.make_fillet();
        for edge in body.edges() {
            make_fillet.add(fillet_radius, &edge);
        }
        make_fillet.build()
    };

    // Create the neck from a cylinder
    let neck_plane = Point::new(0.0, 0.0, height).axis2d_with(Direction::z());
    let neck_radius = thickness / 4.0;
    let neck_height = height / 10.0;

    let neck = make_cylinder(&neck_plane, neck_radius, neck_height);

    // Fuse the body and the neck
    let body = body.fuse(&neck);

    // Hollow out the body, leaving a hole at the top of the neck
    let body = {
        let face_to_remove = body
            .faces()
            .max_by_key(|face| {
                if let Some(plane) = face.surface().as_plane() {
                    OrderedFloat(plane.location().z())
                } else {
                    OrderedFloat(f64::NEG_INFINITY)
                }
            })
            .unwrap();

        body.make_thick_solid()
            .faces_to_remove(&[&face_to_remove])
            .offset(-thickness / 50.0)
            .tolerance(1.0e-3)
            .build()
    };

    // // Add threading to the neck
    // let threading = {
    //     let cylinder1 = CylindricalSurface::new(neck_plane, neck_radius * 0.99);
    //     let cylinder2 = CylindricalSurface::new(neck_plane, neck_radius * 1.05);
    //
    //     let axis2d = Point2D::new(2.0 * PI, neck_height / 2.0)
    //         .axis2d_with(Direction2D::new(2.0 * PI, neck_height / 4.0));
    //
    //     let major = 2.0 * PI;
    //     let minor = neck_height / 10.0;
    //
    //     let ellipse1 = Ellipse2D::new(axis2d, major, minor);
    //     let ellipse2 = Ellipse2D::new(axis2d, major, minor / 4.0);
    //     let arc1 = ellipse1.trim(0.0, PI);
    //     let arc2 = ellipse2.trim(0.0, PI);
    //
    //     let segment = TrimmedCurve2D::line(ellipse1.value(0.0), ellipse1.value(PI));
    //
    //     let threading_wire1 = Wire::new(&[
    //         Edge::new_with_surface(arc1, cylinder1),
    //         Edge::new_with_surface(segment, cylinder1),
    //     ])
    //     .build_curves_3d();
    //     let threading_wire2 = Wire::new(&[
    //         Edge::new_with_surface(arc2, cylinder2),
    //         Edge::new_with_surface(segment, cylinder2),
    //     ])
    //     .build_curves_3d();
    //
    //     Loft::new_solid()
    //         .add_wires(&[threading_wire1, threading_wire2])
    //         .check_compatibility(false)
    //         .build();
    // };
    //
    // let result = Compound::new().add_shapes(&[body, threading]);
}
