#pragma once
#include "BRepBuilderAPI_MakeEdge.hxx"
#include "BRepBuilderAPI_MakeFace.hxx"
#include "BRepBuilderAPI_MakeVertex.hxx"
#include "BRepBuilderAPI_MakeWire.hxx"
#include "BRepBuilderAPI_Transform.hxx"
#include "BRepFilletAPI_MakeFillet.hxx"
#include "BRepPrimAPI_MakePrism.hxx"
#include "BRep_Tool.hxx"
#include "TopExp_Explorer.hxx"
#include "TopoDS_Edge.hxx"
#include "TopoDS_Vertex.hxx"
#include "TopoDS_Wire.hxx"
#include "geom.hpp"
#include <TopoDS.hxx>

namespace occara {
namespace geom {
struct TrimmedCurve;
struct Vector;
struct Transformation;
} // namespace geom
} // namespace occara

namespace occara::shape {

struct Vertex {
  TopoDS_Vertex vertex;

  Vertex(Standard_Real x, Standard_Real y, Standard_Real z);
  void set_coordinates(Standard_Real x, Standard_Real y, Standard_Real z);
  void get_coordinates(double &x, double &y, double &z) const;
};

struct Edge;
struct Shape;

struct FilletBuilder {
  BRepFilletAPI_MakeFillet make_fillet;

  void add_edge(Standard_Real radius, const Edge &edge);
  Shape build();
};

struct Shape {
  TopoDS_Shape shape;

  FilletBuilder make_fillet() const;
};

struct Edge {
  TopoDS_Edge edge;

  Edge(const occara::geom::TrimmedCurve &curve);
  Edge(const TopoDS_Edge &edge);
};

struct EdgeIterator {
  TopExp_Explorer explorer;
  EdgeIterator(const Shape &shape);

  bool more() const;
  Edge next();
};

struct Face {
  TopoDS_Face face;

  Shape extrude(const occara::geom::Vector &vector) const;
};

struct MakeWire;

struct Wire {
  TopoDS_Wire wire;

  Wire(MakeWire &make_wire);
  Wire(const TopoDS_Wire &wire);
  Wire(const Wire &other);
  static Wire clone(const Wire &other);

  Wire transform(const occara::geom::Transformation &transformation) const;
  Face make_face() const;
};

struct MakeWire {
  BRepBuilderAPI_MakeWire make_wire;

  void add_edge(const occara::shape::Edge &edge);
  void add_wire(const occara::shape::Wire &wire);
};

Shape make_cylinder(const occara::geom::Axis2d &axis, Standard_Real radius,
                    Standard_Real height);

} // namespace occara::shape
