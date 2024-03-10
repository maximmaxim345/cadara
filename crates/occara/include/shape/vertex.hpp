#pragma once
#include "TopoDS_Vertex.hxx"
#include <memory>

namespace shape {
struct Vertex {
  TopoDS_Vertex vertex;
  Vertex(double x, double y, double z);

  void set_coordinates(double x, double y, double z);
  void get_coordinates(double &x, double &y, double &z) const;
};

std::unique_ptr<Vertex> vertex_new();
std::unique_ptr<Vertex> vertex_new_with_coordinates(double x, double y,
                                                    double z);
} // namespace shape
