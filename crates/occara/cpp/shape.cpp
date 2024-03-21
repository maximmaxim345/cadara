#include "shape.hpp"
#include "BRepAlgoAPI_Fuse.hxx"
#include "BRepPrimAPI_MakeCylinder.hxx"
#include <BRepLib.hxx>

namespace occara::shape {

Vertex::Vertex(Standard_Real x, Standard_Real y, Standard_Real z)
    : vertex(BRepBuilderAPI_MakeVertex(gp_Pnt(x, y, z))) {}

void Vertex::set_coordinates(Standard_Real x, Standard_Real y,
                             Standard_Real z) {
  vertex = BRepBuilderAPI_MakeVertex(gp_Pnt(x, y, z));
}

void Vertex::get_coordinates(double &x, double &y, double &z) const {
  gp_Pnt point = BRep_Tool::Pnt(vertex);
  x = point.X();
  y = point.Y();
  z = point.Z();
}

ThickSolidBuilder::ThickSolidBuilder(const Shape &shape) : shape(shape.shape) {}

void ThickSolidBuilder::add_face_to_remove(const Face &face) {
  facesToRemove.Append(face.face);
}

void ThickSolidBuilder::set_offset(Standard_Real offset) {
  this->offset = offset;
}

void ThickSolidBuilder::set_tolerance(Standard_Real tolerance) {
  this->tolerance = tolerance;
}

Shape ThickSolidBuilder::build() {
  BRepOffsetAPI_MakeThickSolid make_thick_solid;
  make_thick_solid.MakeThickSolidByJoin(shape, facesToRemove, offset,
                                        tolerance);
  return Shape{make_thick_solid.Shape()};
}

FilletBuilder Shape::make_fillet() const {
  return FilletBuilder{BRepFilletAPI_MakeFillet(shape)};
}

Shape Shape::fuse(const Shape &other) {
  return Shape{BRepAlgoAPI_Fuse(shape, other.shape).Shape()};
}

Edge::Edge(const occara::geom::TrimmedCurve &curve)
    : edge(BRepBuilderAPI_MakeEdge(curve.curve)) {}

Edge::Edge(const TopoDS_Edge &edge) : edge(edge) {}

Edge::Edge(const occara::geom::TrimmedCurve2D &curve,
           const occara::geom::CylindricalSurface &surface)
    : edge(BRepBuilderAPI_MakeEdge(curve.curve, surface.surface)) {}

void FilletBuilder::add_edge(Standard_Real radius, const Edge &edge) {
  make_fillet.Add(radius, edge.edge);
}

Shape FilletBuilder::build() { return Shape{make_fillet.Shape()}; }

EdgeIterator::EdgeIterator(const Shape &shape)
    : explorer(shape.shape, TopAbs_EDGE) {}

bool EdgeIterator::more() const { return explorer.More(); }

Edge EdgeIterator::next() {
  if (explorer.More()) {
    Edge edge{TopoDS::Edge(explorer.Current())};
    explorer.Next();
    return edge;
  }
  throw std::out_of_range("No more edges");
}

FaceIterator::FaceIterator(const Shape &shape)
    : explorer(shape.shape, TopAbs_FACE) {}

bool FaceIterator::more() const { return explorer.More(); }

Face FaceIterator::next() {
  if (explorer.More()) {
    Face face{TopoDS::Face(explorer.Current())};
    explorer.Next();
    return face;
  }
  throw std::out_of_range("No more faces");
}

Shape Face::extrude(const occara::geom::Vector &vector) const {
  return Shape{BRepPrimAPI_MakePrism(face, vector.vector).Shape()};
}

geom::Surface Face::surface() const {
  return geom::Surface{BRep_Tool::Surface(face)};
}

Wire::Wire(WireBuilder &make_wire) : wire(make_wire.make_wire.Wire()) {}

Wire::Wire(const TopoDS_Wire &wire) : wire(wire) {}

Wire::Wire(const Wire &other) : wire(other.wire) {}

Wire Wire::clone(const Wire &other) { return Wire(other.wire); }

Wire Wire::transform(const occara::geom::Transformation &transformation) const {
  BRepBuilderAPI_Transform transform(wire, transformation.transformation);
  return Wire(TopoDS::Wire(transform.Shape()));
}

Face Wire::make_face() const { return Face{BRepBuilderAPI_MakeFace(wire)}; }

void Wire::build_curves_3d() { BRepLib::BuildCurves3d(wire); }

void WireBuilder::add_edge(const occara::shape::Edge &edge) {
  make_wire.Add(edge.edge);
}

void WireBuilder::add_wire(const occara::shape::Wire &wire) {
  make_wire.Add(wire.wire);
}

Shape make_cylinder(const occara::geom::PlaneAxis &axis, Standard_Real radius,
                    Standard_Real height) {
  BRepPrimAPI_MakeCylinder cylinder(axis.axis, radius, height);
  return Shape(cylinder.Shape());
}

Loft::Loft(Standard_Boolean solid) : loft(solid) {}

void Loft::add_wire(const Wire &wire) { loft.AddWire(wire.wire); }

void Loft::check_compatibility(Standard_Boolean check) {
  loft.CheckCompatibility(check);
}

Shape Loft::build() { return Shape(loft.Shape()); }

} // namespace occara::shape
