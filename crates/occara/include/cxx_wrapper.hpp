#pragma once
#include "geom.hpp"
#include "shape.hpp"
#include <memory>

// This file provides wrappers that adapt the existing C++ API to work with cxx
// by returning std::unique_ptr instead of values

namespace occara {
namespace geom {

// Point wrappers
inline std::unique_ptr<Point> Point_create(Standard_Real x, Standard_Real y, Standard_Real z) {
    return std::make_unique<Point>(Point::create(x, y, z));
}

inline std::unique_ptr<Point> Point_clone(const Point& p) {
    return std::make_unique<Point>(p.clone());
}

inline void Point_get_coordinates(const Point& p, Standard_Real& x, Standard_Real& y, Standard_Real& z) {
    p.get_coordinates(x, y, z);
}

inline Standard_Real Point_x(const Point& p) { return p.x(); }
inline Standard_Real Point_y(const Point& p) { return p.y(); }
inline Standard_Real Point_z(const Point& p) { return p.z(); }

// Point2D wrappers
inline std::unique_ptr<Point2D> Point2D_create(Standard_Real x, Standard_Real y) {
    return std::make_unique<Point2D>(Point2D::create(x, y));
}

inline std::unique_ptr<Point2D> Point2D_clone(const Point2D& p) {
    return std::make_unique<Point2D>(p.clone());
}

inline void Point2D_get_coordinates(const Point2D& p, Standard_Real& x, Standard_Real& y) {
    p.get_coordinates(x, y);
}

inline Standard_Real Point2D_x(const Point2D& p) { return p.x(); }
inline Standard_Real Point2D_y(const Point2D& p) { return p.y(); }

// Vector wrappers
inline std::unique_ptr<Vector> Vector_create(Standard_Real x, Standard_Real y, Standard_Real z) {
    return std::make_unique<Vector>(Vector::create(x, y, z));
}

inline std::unique_ptr<Vector> Vector_clone(const Vector& v) {
    return std::make_unique<Vector>(v.clone());
}

// Direction wrappers
inline std::unique_ptr<Direction> Direction_create(Standard_Real x, Standard_Real y, Standard_Real z) {
    return std::make_unique<Direction>(Direction::create(x, y, z));
}

inline std::unique_ptr<Direction> Direction_clone(const Direction& d) {
    return std::make_unique<Direction>(d.clone());
}

inline void Direction_get_components(const Direction& d, Standard_Real& x, Standard_Real& y, Standard_Real& z) {
    d.get_components(x, y, z);
}

// Direction2D wrappers
inline std::unique_ptr<Direction2D> Direction2D_create(Standard_Real x, Standard_Real y) {
    return std::make_unique<Direction2D>(Direction2D::create(x, y));
}

inline std::unique_ptr<Direction2D> Direction2D_clone(const Direction2D& d) {
    return std::make_unique<Direction2D>(d.clone());
}

inline void Direction2D_get_components(const Direction2D& d, Standard_Real& x, Standard_Real& y) {
    d.get_components(x, y);
}

// Axis wrappers
inline std::unique_ptr<Axis> Axis_create(const Point& origin, const Direction& direction) {
    return std::make_unique<Axis>(Axis::create(origin, direction));
}

inline std::unique_ptr<Axis> Axis_clone(const Axis& a) {
    return std::make_unique<Axis>(a.clone());
}

inline std::unique_ptr<Point> Axis_location(const Axis& a) {
    return std::make_unique<Point>(a.location());
}

inline std::unique_ptr<Direction> Axis_direction(const Axis& a) {
    return std::make_unique<Direction>(a.direction());
}

// Axis2D wrappers
inline std::unique_ptr<Axis2D> Axis2D_create(const Point2D& origin, const Direction2D& direction) {
    return std::make_unique<Axis2D>(Axis2D::create(origin, direction));
}

inline std::unique_ptr<Axis2D> Axis2D_clone(const Axis2D& a) {
    return std::make_unique<Axis2D>(a.clone());
}

inline std::unique_ptr<Point2D> Axis2D_location(const Axis2D& a) {
    return std::make_unique<Point2D>(a.location());
}

inline std::unique_ptr<Direction2D> Axis2D_direction(const Axis2D& a) {
    return std::make_unique<Direction2D>(a.direction());
}

// PlaneAxis wrappers
inline std::unique_ptr<PlaneAxis> PlaneAxis_create(const Point& origin, const Direction& direction) {
    return std::make_unique<PlaneAxis>(PlaneAxis::create(origin, direction));
}

inline std::unique_ptr<PlaneAxis> PlaneAxis_clone(const PlaneAxis& a) {
    return std::make_unique<PlaneAxis>(a.clone());
}

inline std::unique_ptr<Point> PlaneAxis_location(const PlaneAxis& a) {
    return std::make_unique<Point>(a.location());
}

inline std::unique_ptr<Direction> PlaneAxis_direction(const PlaneAxis& a) {
    return std::make_unique<Direction>(a.direction());
}

// SpaceAxis wrappers
inline std::unique_ptr<SpaceAxis> SpaceAxis_create(const Point& origin, const Direction& direction) {
    return std::make_unique<SpaceAxis>(SpaceAxis::create(origin, direction));
}

inline std::unique_ptr<SpaceAxis> SpaceAxis_clone(const SpaceAxis& a) {
    return std::make_unique<SpaceAxis>(a.clone());
}

inline std::unique_ptr<Point> SpaceAxis_location(const SpaceAxis& a) {
    return std::make_unique<Point>(a.location());
}

inline std::unique_ptr<Direction> SpaceAxis_direction(const SpaceAxis& a) {
    return std::make_unique<Direction>(a.direction());
}

// TrimmedCurve wrappers
inline std::unique_ptr<TrimmedCurve> TrimmedCurve_arc_of_circle(const Point& p1, const Point& p2, const Point& p3) {
    return std::make_unique<TrimmedCurve>(TrimmedCurve::arc_of_circle(p1, p2, p3));
}

inline std::unique_ptr<TrimmedCurve> TrimmedCurve_line(const Point& p1, const Point& p2) {
    return std::make_unique<TrimmedCurve>(TrimmedCurve::line(p1, p2));
}

inline std::unique_ptr<TrimmedCurve> TrimmedCurve_clone(const TrimmedCurve& c) {
    return std::make_unique<TrimmedCurve>(c.clone());
}

// TrimmedCurve2D wrappers
inline std::unique_ptr<TrimmedCurve2D> TrimmedCurve2D_line(const Point2D& p1, const Point2D& p2) {
    return std::make_unique<TrimmedCurve2D>(TrimmedCurve2D::line(p1, p2));
}

inline std::unique_ptr<TrimmedCurve2D> TrimmedCurve2D_clone(const TrimmedCurve2D& c) {
    return std::make_unique<TrimmedCurve2D>(c.clone());
}

// Curve2D wrappers
inline std::unique_ptr<Curve2D> Curve2D_from_trimmed_curve2d(const TrimmedCurve2D& curve) {
    return std::make_unique<Curve2D>(Curve2D::from_trimmed_curve2d(curve));
}

inline std::unique_ptr<Curve2D> Curve2D_clone(const Curve2D& c) {
    return std::make_unique<Curve2D>(c.clone());
}

inline std::unique_ptr<TrimmedCurve2D> Curve2D_trim(const Curve2D& c, Standard_Real u1, Standard_Real u2) {
    return std::make_unique<TrimmedCurve2D>(c.trim(u1, u2));
}

// Ellipse2D wrappers
inline std::unique_ptr<Ellipse2D> Ellipse2D_create(const Axis2D& axis, Standard_Real major_radius, Standard_Real minor_radius) {
    return std::make_unique<Ellipse2D>(Ellipse2D::create(axis, major_radius, minor_radius));
}

inline std::unique_ptr<Ellipse2D> Ellipse2D_clone(const Ellipse2D& e) {
    return std::make_unique<Ellipse2D>(e.clone());
}

inline std::unique_ptr<Point2D> Ellipse2D_value(const Ellipse2D& e, Standard_Real u) {
    return std::make_unique<Point2D>(e.value(u));
}

inline std::unique_ptr<Curve2D> Ellipse2D_curve(const Ellipse2D& e) {
    return std::make_unique<Curve2D>(e.curve());
}

// Plane wrappers
inline std::unique_ptr<Plane> Plane_clone(const Plane& p) {
    return std::make_unique<Plane>(p.clone());
}

inline std::unique_ptr<Point> Plane_location(const Plane& p) {
    return std::make_unique<Point>(p.location());
}

// Surface wrappers
inline std::unique_ptr<Surface> Surface_from_cylindrical_surface(const CylindricalSurface& surface) {
    return std::make_unique<Surface>(Surface::from_cylindrical_surface(surface));
}

inline std::unique_ptr<Surface> Surface_clone(const Surface& s) {
    return std::make_unique<Surface>(s.clone());
}

inline bool Surface_is_plane(const Surface& s) {
    return s.is_plane();
}

inline std::unique_ptr<Plane> Surface_as_plane(const Surface& s) {
    return std::make_unique<Plane>(s.as_plane());
}

// Transformation wrappers
inline std::unique_ptr<Transformation> Transformation_new() {
    return std::make_unique<Transformation>();
}

inline std::unique_ptr<Transformation> Transformation_clone(const Transformation& t) {
    return std::make_unique<Transformation>(t.clone());
}

inline void Transformation_mirror(Transformation& t, const Axis& axis) {
    t.mirror(axis);
}

// CylindricalSurface wrappers
inline std::unique_ptr<CylindricalSurface> CylindricalSurface_create(const PlaneAxis& axis, Standard_Real radius) {
    return std::make_unique<CylindricalSurface>(CylindricalSurface::create(axis, radius));
}

inline std::unique_ptr<CylindricalSurface> CylindricalSurface_clone(const CylindricalSurface& s) {
    return std::make_unique<CylindricalSurface>(s.clone());
}

} // namespace geom

namespace shape {

// Vertex wrappers
inline std::unique_ptr<Vertex> Vertex_create(const geom::Point& point) {
    return std::make_unique<Vertex>(Vertex::create(point));
}

inline std::unique_ptr<Vertex> Vertex_clone(const Vertex& v) {
    return std::make_unique<Vertex>(v.clone());
}

inline std::unique_ptr<geom::Point> Vertex_point(const Vertex& v) {
    return std::make_unique<geom::Point>(v.point());
}

// Shape wrappers
inline std::unique_ptr<Shape> Shape_clone(const Shape& s) {
    return std::make_unique<Shape>(s.clone());
}

inline std::unique_ptr<FilletBuilder> Shape_fillet(const Shape& s) {
    return std::make_unique<FilletBuilder>(s.fillet());
}

inline std::unique_ptr<Shape> Shape_fuse(const Shape& s, const Shape& other) {
    return std::make_unique<Shape>(s.fuse(other));
}

inline std::unique_ptr<Shape> Shape_subtract(const Shape& s, const Shape& other) {
    return std::make_unique<Shape>(s.subtract(other));
}

inline std::unique_ptr<Shape> Shape_intersect(const Shape& s, const Shape& other) {
    return std::make_unique<Shape>(s.intersect(other));
}

inline std::unique_ptr<Shape> Shape_cylinder(const geom::PlaneAxis& axis, Standard_Real radius, Standard_Real height) {
    return std::make_unique<Shape>(Shape::cylinder(axis, radius, height));
}

inline std::unique_ptr<Mesh> Shape_mesh(const Shape& s) {
    return std::make_unique<Mesh>(s.mesh());
}

inline uint32_t Shape_shape_type(const Shape& s) {
    return static_cast<uint32_t>(s.shape_type());
}

inline Standard_Boolean Shape_is_null(const Shape& s) {
    return s.is_null();
}

inline Standard_Boolean Shape_is_closed(const Shape& s) {
    return s.is_closed();
}

inline Standard_Real Shape_mass(const Shape& s) {
    return s.mass();
}

// FilletBuilder wrappers
inline std::unique_ptr<FilletBuilder> FilletBuilder_clone(const FilletBuilder& f) {
    return std::make_unique<FilletBuilder>(f.clone());
}

inline void FilletBuilder_add_edge(FilletBuilder& f, Standard_Real radius, const Edge& edge) {
    f.add_edge(radius, edge);
}

inline std::unique_ptr<Shape> FilletBuilder_build(FilletBuilder& f) {
    return std::make_unique<Shape>(f.build());
}

// ShellBuilder wrappers
inline std::unique_ptr<ShellBuilder> ShellBuilder_create(const Shape& shape) {
    return std::make_unique<ShellBuilder>(ShellBuilder::create(shape));
}

inline std::unique_ptr<ShellBuilder> ShellBuilder_clone(const ShellBuilder& s) {
    return std::make_unique<ShellBuilder>(s.clone());
}

inline void ShellBuilder_add_face_to_remove(ShellBuilder& s, const Face& face) {
    s.add_face_to_remove(face);
}

inline void ShellBuilder_set_offset(ShellBuilder& s, Standard_Real offset) {
    s.set_offset(offset);
}

inline void ShellBuilder_set_tolerance(ShellBuilder& s, Standard_Real tolerance) {
    s.set_tolerance(tolerance);
}

inline std::unique_ptr<Shape> ShellBuilder_build(ShellBuilder& s) {
    return std::make_unique<Shape>(s.build());
}

// Edge wrappers
inline std::unique_ptr<Edge> Edge_from_curve(const geom::TrimmedCurve& curve) {
    return std::make_unique<Edge>(Edge::from_curve(curve));
}

inline std::unique_ptr<Edge> Edge_clone(const Edge& e) {
    return std::make_unique<Edge>(e.clone());
}

inline std::unique_ptr<Edge> Edge_from_2d_curve(const geom::Curve2D& curve, const geom::Surface& surface) {
    return std::make_unique<Edge>(Edge::from_2d_curve(curve, surface));
}

// EdgeIterator wrappers
inline std::unique_ptr<EdgeIterator> EdgeIterator_create(const Shape& shape) {
    // Use the static create method and move the result
    auto it_value = EdgeIterator::create(shape);
    // Create unique_ptr by copying (EdgeIterator should be copyable)
    return std::make_unique<EdgeIterator>(std::move(it_value));
}

inline std::unique_ptr<EdgeIterator> EdgeIterator_clone(const EdgeIterator& it) {
    return std::make_unique<EdgeIterator>(it.clone());
}

inline bool EdgeIterator_more(const EdgeIterator& it) {
    return it.more();
}

inline std::unique_ptr<Edge> EdgeIterator_next(EdgeIterator& it) {
    return std::make_unique<Edge>(it.next());
}

// Face wrappers
inline std::unique_ptr<Face> Face_clone(const Face& f) {
    return std::make_unique<Face>(f.clone());
}

inline std::unique_ptr<Shape> Face_extrude(const Face& f, const geom::Vector& vector) {
    return std::make_unique<Shape>(f.extrude(vector));
}

inline std::unique_ptr<geom::Surface> Face_surface(const Face& f) {
    return std::make_unique<geom::Surface>(f.surface());
}

// FaceIterator wrappers
inline std::unique_ptr<FaceIterator> FaceIterator_create(const Shape& shape) {
    return std::make_unique<FaceIterator>(FaceIterator::create(shape));
}

inline std::unique_ptr<FaceIterator> FaceIterator_clone(const FaceIterator& it) {
    return std::make_unique<FaceIterator>(it.clone());
}

inline bool FaceIterator_more(const FaceIterator& it) {
    return it.more();
}

inline std::unique_ptr<Face> FaceIterator_next(FaceIterator& it) {
    return std::make_unique<Face>(it.next());
}

// Wire wrappers
inline std::unique_ptr<Wire> Wire_create(WireBuilder& make_wire) {
    return std::make_unique<Wire>(Wire::create(make_wire));
}

inline std::unique_ptr<Wire> Wire_clone(const Wire& w) {
    return std::make_unique<Wire>(w.clone());
}

inline std::unique_ptr<Wire> Wire_transform(const Wire& w, const geom::Transformation& transformation) {
    return std::make_unique<Wire>(w.transform(transformation));
}

inline std::unique_ptr<Face> Wire_face(const Wire& w) {
    return std::make_unique<Face>(w.face());
}

inline void Wire_build_curves_3d(Wire& w) {
    w.build_curves_3d();
}

// WireBuilder wrappers
inline std::unique_ptr<WireBuilder> WireBuilder_new() {
    return std::make_unique<WireBuilder>();
}

inline std::unique_ptr<WireBuilder> WireBuilder_clone(const WireBuilder& w) {
    return std::make_unique<WireBuilder>(w.clone());
}

inline void WireBuilder_add_edge(WireBuilder& w, const Edge& edge) {
    w.add_edge(edge);
}

inline void WireBuilder_add_wire(WireBuilder& w, const Wire& wire) {
    w.add_wire(wire);
}

// Loft wrappers
inline std::unique_ptr<Loft> Loft_create_solid() {
    return std::make_unique<Loft>(Loft::create_solid());
}

inline std::unique_ptr<Loft> Loft_clone(const Loft& l) {
    return std::make_unique<Loft>(l.clone());
}

inline void Loft_add_wire(Loft& l, const Wire& wire) {
    l.add_wire(wire);
}

inline void Loft_ensure_wire_compatibility(Loft& l, Standard_Boolean check) {
    l.ensure_wire_compatibility(check);
}

inline std::unique_ptr<Shape> Loft_build(Loft& l) {
    return std::make_unique<Shape>(l.build());
}

// Compound wrappers
inline std::unique_ptr<Compound> Compound_new() {
    return std::make_unique<Compound>();
}

inline void Compound_init(Compound& c) {
    c.init();
}

inline void Compound_add_shape(Compound& c, const Shape& shape) {
    c.add_shape(shape);
}

inline std::unique_ptr<Shape> Compound_build(Compound& c) {
    return std::make_unique<Shape>(c.build());
}

// Mesh wrappers
inline size_t Mesh_indices_size(const Mesh& m) {
    return m.indices_size();
}

inline size_t Mesh_vertices_size(const Mesh& m) {
    return m.vertices_size();
}

inline size_t Mesh_indices_at(const Mesh& m, size_t index) {
    return m.indices_at(index);
}

inline std::unique_ptr<geom::Point> Mesh_vertices_at(const Mesh& m, size_t index) {
    return std::make_unique<geom::Point>(m.vertices_at(index));
}

} // namespace shape
} // namespace occara

// MakeBottle wrapper (in global namespace)
#include "internal/MakeBottle.hpp"

inline std::unique_ptr<occara::shape::Shape> MakeBottle_wrapper(Standard_Real theWidth, Standard_Real theHeight, Standard_Real theThickness) {
    return std::make_unique<occara::shape::Shape>(std::move(MakeBottle(theWidth, theHeight, theThickness)));
}
