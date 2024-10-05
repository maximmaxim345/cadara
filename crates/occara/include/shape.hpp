#pragma once
#include "BRepBuilderAPI_MakeEdge.hxx"
#include "BRepBuilderAPI_MakeFace.hxx"
#include "BRepBuilderAPI_MakeVertex.hxx"
#include "BRepBuilderAPI_MakeWire.hxx"
#include "BRepBuilderAPI_Transform.hxx"
#include "BRepFilletAPI_MakeFillet.hxx"
#include "BRepMesh_IncrementalMesh.hxx"
#include "BRepOffsetAPI_MakeThickSolid.hxx"
#include "BRepOffsetAPI_ThruSections.hxx"
#include "BRepPrimAPI_MakePrism.hxx"
#include "BRep_Tool.hxx"
#include "IMeshData_Status.hxx"
#include "IMeshTools_Parameters.hxx"
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
struct Mesh;

struct Vertex {
  TopoDS_Vertex vertex;

  static Vertex create(const geom::Point &point);
  Vertex clone() const;

  geom::Point point() const;
};

struct FilletBuilder {
  BRepFilletAPI_MakeFillet make_fillet;

  FilletBuilder clone() const;

  void add_edge(Standard_Real radius, const Edge &edge);
  Shape build();
};

struct ShellBuilder {
  TopoDS_Shape shape;
  TopTools_ListOfShape faces_to_remove = TopTools_ListOfShape();
  Standard_Real tolerance = 1.e-3;
  Standard_Real offset = 0.0;

  static ShellBuilder create(const Shape &shape);
  ShellBuilder clone() const;

  void add_face_to_remove(const Face &face);
  void set_offset(Standard_Real offset);
  void set_tolerance(Standard_Real tolerance);
  Shape build();
};

// This is equal to TopAbs_ShapeEnum
enum class ShapeType {
  Compound,
  CompoundSolid,
  Solid,
  Shell,
  Face,
  Wire,
  Edge,
  Vertex,
  Shape
};

struct Shape {
  TopoDS_Shape shape;

  Shape clone() const;

  FilletBuilder fillet() const;
  Shape fuse(const Shape &other) const;
  static Shape cylinder(const occara::geom::PlaneAxis &axis,
                        Standard_Real radius, Standard_Real height);
  Mesh mesh() const;
  ShapeType shape_type() const;
};

struct Edge {
  TopoDS_Edge edge;

  static Edge from_curve(const occara::geom::TrimmedCurve &curve);
  Edge clone() const;
  static Edge from_2d_curve(const occara::geom::Curve2D &curve,
                            const occara::geom::Surface &surface);
};

struct EdgeIterator {
  TopExp_Explorer explorer;

  static EdgeIterator create(const Shape &shape);
  EdgeIterator clone() const;

  bool more() const;
  Edge next();
};

struct Face {
  TopoDS_Face face;

  Face clone() const;

  Shape extrude(const occara::geom::Vector &vector) const;
  geom::Surface surface() const;
};

struct FaceIterator {
  TopExp_Explorer explorer;

  static FaceIterator create(const Shape &shape);
  FaceIterator clone() const;

  bool more() const;
  Face next();
};

struct Wire {
  TopoDS_Wire wire;

  static Wire create(WireBuilder &make_wire);
  Wire clone() const;

  Wire transform(const occara::geom::Transformation &transformation) const;
  Face face() const;
  void build_curves_3d();
};

struct WireBuilder {
  BRepBuilderAPI_MakeWire make_wire;

  WireBuilder clone() const;

  void add_edge(const occara::shape::Edge &edge);
  void add_wire(const occara::shape::Wire &wire);
};

struct Loft {
  BRepOffsetAPI_ThruSections loft;

  static Loft create_solid();
  Loft clone() const;

  void add_wire(const Wire &wire);
  void ensure_wire_compatibility(Standard_Boolean check);
  Shape build();
};

struct Compound {
  TopoDS_Compound compound;
  BRep_Builder builder;

  Compound();
  static Compound create();
  // Disable copy and move, since we are self-referential
  Compound(const Compound &) = delete;
  Compound &operator=(const Compound &) = delete;
  Compound(Compound &&) = delete;
  Compound &operator=(Compound &&) = delete;

  void add_shape(const Shape &shape);
  Shape build();
};

struct Mesh {
  // FIXME: This is quite inefficient but works while using autocxx.
  // Later we should solve this by manually creating a binding for this class
  // (and dependencies like geom::Point) using cxx. So rust code can directly
  // use the data without copying it.
  std::vector<size_t> indices;
  std::vector<geom::Point> vertices;

  size_t indices_size() const { return indices.size(); }

  size_t vertices_size() const { return vertices.size(); }

  size_t indices_at(size_t index) const { return indices[index]; }

  geom::Point vertices_at(size_t index) const { return vertices[index]; }
};

} // namespace occara::shape
