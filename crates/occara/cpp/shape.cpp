#include "shape.hpp"

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

FilletBuilder Shape::make_fillet() const {
  return FilletBuilder{BRepFilletAPI_MakeFillet(shape)};
}

Edge::Edge(const occara::geom::TrimmedCurve &curve)
    : edge(BRepBuilderAPI_MakeEdge(curve.curve)) {}

Edge::Edge(const TopoDS_Edge &edge) : edge(edge) {}

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

Shape Face::extrude(const occara::geom::Vector &vector) const {
  return Shape{BRepPrimAPI_MakePrism(face, vector.vector).Shape()};
}

Wire::Wire(MakeWire &make_wire) : wire(make_wire.make_wire.Wire()) {}

Wire::Wire(const TopoDS_Wire &wire) : wire(wire) {}

Wire::Wire(const Wire &other) : wire(other.wire) {}

Wire Wire::clone(const Wire &other) { return Wire(other.wire); }

Wire Wire::transform(const occara::geom::Transformation &transformation) const {
  BRepBuilderAPI_Transform transform(wire, transformation.transformation);
  return Wire(TopoDS::Wire(transform.Shape()));
}

Face Wire::make_face() const { return Face{BRepBuilderAPI_MakeFace(wire)}; }

void MakeWire::add_edge(const occara::shape::Edge &edge) {
  make_wire.Add(edge.edge);
}

void MakeWire::add_wire(const occara::shape::Wire &wire) {
  make_wire.Add(wire.wire);
}

} // namespace occara::shape
