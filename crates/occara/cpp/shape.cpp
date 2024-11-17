#include "shape.hpp"
#include "BRepAlgoAPI_Common.hxx"
#include "BRepAlgoAPI_Cut.hxx"
#include "BRepAlgoAPI_Fuse.hxx"
#include "BRepPrimAPI_MakeCylinder.hxx"
#include <BRepGProp.hxx>
#include <BRepLib.hxx>
#include <GProp_GProps.hxx>

namespace occara::shape {

// Vertex

Vertex Vertex::create(const geom::Point &point) {
  return Vertex{BRepBuilderAPI_MakeVertex(point.point)};
}

Vertex Vertex::clone() const { return *this; }

geom::Point Vertex::point() const {
  return geom::Point{BRep_Tool::Pnt(vertex)};
}

// FilletBuilder

FilletBuilder FilletBuilder::clone() const { return *this; }

void FilletBuilder::add_edge(Standard_Real radius, const Edge &edge) {
  make_fillet.Add(radius, edge.edge);
}

Shape FilletBuilder::build() { return Shape{make_fillet.Shape()}; }

// ShellBuilder

ShellBuilder ShellBuilder::create(const Shape &shape) {
  return ShellBuilder{shape.shape};
}

ShellBuilder ShellBuilder::clone() const { return *this; }

void ShellBuilder::add_face_to_remove(const Face &face) {
  faces_to_remove.Append(face.face);
}

void ShellBuilder::set_offset(Standard_Real offset) { this->offset = offset; }

void ShellBuilder::set_tolerance(Standard_Real tolerance) {
  this->tolerance = tolerance;
}

Shape ShellBuilder::build() {
  BRepOffsetAPI_MakeThickSolid make_thick_solid;
  make_thick_solid.MakeThickSolidByJoin(shape, faces_to_remove, offset,
                                        tolerance);
  return Shape{make_thick_solid.Shape()};
}

// Shape

Shape Shape::clone() const { return *this; }

FilletBuilder Shape::fillet() const {
  return FilletBuilder{BRepFilletAPI_MakeFillet(shape)};
}

Shape Shape::fuse(const Shape &other) const {
  return Shape{BRepAlgoAPI_Fuse(shape, other.shape).Shape()};
}

Shape Shape::subtract(const Shape &other) const {
  return Shape{BRepAlgoAPI_Cut(shape, other.shape).Shape()};
}

Shape Shape::intersect(const Shape &other) const {
  return Shape{BRepAlgoAPI_Common(shape, other.shape).Shape()};
}

Shape Shape::cylinder(const occara::geom::PlaneAxis &axis, Standard_Real radius,
                      Standard_Real height) {
  BRepPrimAPI_MakeCylinder cylinder(axis.axis, radius, height);
  return Shape{cylinder.Shape()};
}

Mesh Shape::mesh() const {
  // FIXME: this implementation is a proof of concept and needs improvements
  // For a better implementation, see
  // crates/opencascade-sys/occt_source/samples/mfc/standard/01_Geometry/src/GeometryDoc.cpp
  // Mesh parameters
  IMeshTools_Parameters meshParams;
  meshParams.Deflection = 0.01;
  meshParams.Angle = 0.5;
  meshParams.Relative = Standard_False;
  meshParams.InParallel = Standard_True;
  meshParams.MinSize = Precision::Confusion();
  meshParams.InternalVerticesMode = Standard_True;
  meshParams.ControlSurfaceDeflection = Standard_True;

  // Perform meshing
  BRepMesh_IncrementalMesh mesher(shape, meshParams);

  // Collect vertices and indices
  std::vector<geom::Point> vertices;
  std::vector<size_t> indices;

  TopExp_Explorer faceExplorer(shape, TopAbs_FACE);
  for (; faceExplorer.More(); faceExplorer.Next()) {
    TopoDS_Face face = TopoDS::Face(faceExplorer.Current());
    TopLoc_Location loc;
    Handle(Poly_Triangulation) triangulation =
        BRep_Tool::Triangulation(face, loc);

    if (triangulation.IsNull()) {
      continue;
    }

    // Collect triangle indices
    for (int i = 1; i <= triangulation->NbTriangles(); ++i) {
      int n1, n2, n3;
      triangulation->Triangle(i).Get(n1, n2, n3);
      auto p1 = triangulation->Node(n1).Transformed(loc.Transformation());
      auto p2 = triangulation->Node(n2).Transformed(loc.Transformation());
      auto p3 = triangulation->Node(n3).Transformed(loc.Transformation());
      vertices.push_back(geom::Point::create(p1.X(), p1.Y(), p1.Z()));
      indices.push_back(vertices.size() - 1);
      vertices.push_back(geom::Point::create(p2.X(), p2.Y(), p2.Z()));
      indices.push_back(vertices.size() - 1);
      vertices.push_back(geom::Point::create(p3.X(), p3.Y(), p3.Z()));
      indices.push_back(vertices.size() - 1);
    }
  }

  return Mesh{
      indices,
      vertices,
  };
}

ShapeType Shape::shape_type() const {
  return static_cast<ShapeType>(shape.ShapeType());
}

Standard_Boolean Shape::is_null() const { return shape.IsNull(); }

Standard_Boolean Shape::is_closed() const { return shape.Closed(); }

Standard_Real Shape::mass() const {
  GProp_GProps props;
  BRepGProp::VolumeProperties(shape, props);
  return props.Mass();
}

// Edge

Edge Edge::from_curve(const occara::geom::TrimmedCurve &curve) {
  return Edge{BRepBuilderAPI_MakeEdge(curve.curve)};
}

Edge Edge::from_2d_curve(const occara::geom::Curve2D &curve,
                         const occara::geom::Surface &surface) {
  return Edge{BRepBuilderAPI_MakeEdge(curve.curve, surface.surface)};
}

Edge Edge::clone() const { return *this; }

// EdgeIterator

EdgeIterator EdgeIterator::create(const Shape &shape) {
  return EdgeIterator{TopExp_Explorer(shape.shape, TopAbs_EDGE)};
}

EdgeIterator EdgeIterator::clone() const { return *this; }

bool EdgeIterator::more() const { return explorer.More(); }

Edge EdgeIterator::next() {
  Edge edge{TopoDS::Edge(explorer.Current())};
  // We ensure in rust that the next element exists before calling next
  explorer.Next();
  return edge;
}

// Face

Face Face::clone() const { return *this; }

Shape Face::extrude(const occara::geom::Vector &vector) const {
  return Shape{BRepPrimAPI_MakePrism(face, vector.vector).Shape()};
}

geom::Surface Face::surface() const {
  return geom::Surface{BRep_Tool::Surface(face)};
}

// FaceIterator

FaceIterator FaceIterator::create(const Shape &shape) {
  return FaceIterator{TopExp_Explorer(shape.shape, TopAbs_FACE)};
}

FaceIterator FaceIterator::clone() const { return *this; }

bool FaceIterator::more() const { return explorer.More(); }

Face FaceIterator::next() {
  Face face{TopoDS::Face(explorer.Current())};
  // We ensure in rust that the next element exists before calling next
  explorer.Next();
  return face;
}

// Wire

Wire Wire::create(WireBuilder &make_wire) {
  return Wire{make_wire.make_wire.Wire()};
}

Wire Wire::clone() const { return *this; }

Wire Wire::transform(const occara::geom::Transformation &transformation) const {
  BRepBuilderAPI_Transform transform(wire, transformation.transformation);
  return Wire{TopoDS::Wire(transform.Shape())};
}

Face Wire::face() const { return Face{BRepBuilderAPI_MakeFace(wire)}; }

void Wire::build_curves_3d() { BRepLib::BuildCurves3d(wire); }

// WireBuilder

WireBuilder WireBuilder::clone() const { return *this; }

void WireBuilder::add_edge(const occara::shape::Edge &edge) {
  make_wire.Add(edge.edge);
}

void WireBuilder::add_wire(const occara::shape::Wire &wire) {
  make_wire.Add(wire.wire);
}

// Loft

Loft Loft::create_solid() {
  return Loft{BRepOffsetAPI_ThruSections(Standard_True)};
}

Loft Loft::clone() const { return *this; }

void Loft::add_wire(const Wire &wire) { loft.AddWire(wire.wire); }

void Loft::ensure_wire_compatibility(Standard_Boolean check) {
  loft.CheckCompatibility(check);
}

Shape Loft::build() { return Shape{loft.Shape()}; }

// Compound

// Since Compound is self-referential, we require a separate initialization
// step.
// TODO: use constructors instead when supported with autocxx on wasm
void Compound::init() { builder.MakeCompound(compound); }

void Compound::add_shape(const Shape &shape) {
  builder.Add(compound, shape.shape);
}

Shape Compound::build() { return Shape{compound}; }

} // namespace occara::shape
