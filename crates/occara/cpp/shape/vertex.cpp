#include "shape/vertex.hpp"
#include "BRepBuilderAPI_MakeVertex.hxx"
#include "BRep_Tool.hxx"
#include "gp_Pnt.hxx"

namespace shape {

Vertex::Vertex(double x, double y, double z) {
  gp_Pnt point(x, y, z);
  vertex = BRepBuilderAPI_MakeVertex(point);
}

void Vertex::set_coordinates(double x, double y, double z) {
  gp_Pnt point(x, y, z);
  vertex = BRepBuilderAPI_MakeVertex(point);
}

void Vertex::get_coordinates(double &x, double &y, double &z) const {
  gp_Pnt point = BRep_Tool::Pnt(vertex);
  x = point.X();
  y = point.Y();
  z = point.Z();
}

std::unique_ptr<Vertex> vertex_new() {
  return std::make_unique<Vertex>(0, 0, 0);
}

std::unique_ptr<Vertex> vertex_new_with_coordinates(double x, double y,
                                                    double z) {
  return std::make_unique<Vertex>(x, y, z);
}

} // namespace shape
