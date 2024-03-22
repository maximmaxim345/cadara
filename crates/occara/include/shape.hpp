#pragma once
#include "BRepBuilderAPI_MakeEdge.hxx"
#include "BRepBuilderAPI_MakeFace.hxx"
#include "BRepBuilderAPI_MakeVertex.hxx"
#include "BRepBuilderAPI_MakeWire.hxx"
#include "BRepBuilderAPI_Transform.hxx"
#include "BRepFilletAPI_MakeFillet.hxx"
#include "BRepOffsetAPI_MakeThickSolid.hxx"
#include "BRepOffsetAPI_ThruSections.hxx"
#include "BRepPrimAPI_MakePrism.hxx"
#include "BRep_Tool.hxx"
#include "TopExp_Explorer.hxx"
#include "TopoDS.hxx"
#include "TopoDS_Edge.hxx"
#include "TopoDS_Vertex.hxx"
#include "TopoDS_Wire.hxx"
#include "geom.hpp"

namespace occara::shape {

// Forward declarations
struct Vertex;
struct FilletBuilder;
struct ShellBuilder;
struct Shape;
struct Edge;
struct EdgeIterator;
struct Face;
struct FaceIterator;
struct Wire;
struct WireBuilder;
struct Loft;
struct Compound;

struct Vertex {
  TopoDS_Vertex vertex;

  Vertex(Standard_Real x, Standard_Real y, Standard_Real z);

  void set_coordinates(Standard_Real x, Standard_Real y, Standard_Real z);
  void get_coordinates(double &x, double &y, double &z) const;
};

struct FilletBuilder {
  BRepFilletAPI_MakeFillet make_fillet;

  void add_edge(Standard_Real radius, const Edge &edge);
  Shape build();
};

struct ShellBuilder {
  TopoDS_Shape shape;
  TopTools_ListOfShape faces_to_remove;
  Standard_Real tolerance = 1.e-3;
  Standard_Real offset = 0.0;

  ShellBuilder(const Shape &shape);

  void add_face_to_remove(const Face &face);
  void set_offset(Standard_Real offset);
  void set_tolerance(Standard_Real tolerance);
  Shape build();
};

struct Shape {
  TopoDS_Shape shape;

  FilletBuilder fillet() const;
  Shape fuse(const Shape &other) const;
  static Shape cylinder(const occara::geom::PlaneAxis &axis,
                        Standard_Real radius, Standard_Real height);
};

struct Edge {
  TopoDS_Edge edge;

  Edge(const occara::geom::TrimmedCurve &curve);
  Edge(const TopoDS_Edge &edge);
  Edge(const occara::geom::TrimmedCurve2D &curve,
       const occara::geom::CylindricalSurface &surface);
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
  geom::Surface surface() const;
};

struct FaceIterator {
  TopExp_Explorer explorer;

  FaceIterator(const Shape &shape);

  bool more() const;
  Face next();
};

struct Wire {
  TopoDS_Wire wire;

  Wire(WireBuilder &make_wire);
  Wire(const TopoDS_Wire &wire);
  Wire(const Wire &other);

  static Wire clone(const Wire &other);
  Wire transform(const occara::geom::Transformation &transformation) const;
  Face face() const;
  void build_curves_3d();
};

struct WireBuilder {
  BRepBuilderAPI_MakeWire make_wire;

  void add_edge(const occara::shape::Edge &edge);
  void add_wire(const occara::shape::Wire &wire);
};

struct Loft {
  BRepOffsetAPI_ThruSections loft;

  Loft(Standard_Boolean solid);

  void add_wire(const Wire &wire);
  void ensure_wire_compatibility(Standard_Boolean check);
  Shape build();
};

struct Compound {
  TopoDS_Compound compound;
  BRep_Builder builder;

  Compound();

  void add_shape(const Shape &shape);
  Shape build();
};

} // namespace occara::shape
