#include "geom.hpp"
#include "GCE2d_MakeSegment.hxx"

namespace occara::geom {

// Point

Point Point::create(Standard_Real x, Standard_Real y, Standard_Real z) {
  return Point{gp_Pnt(x, y, z)};
}

Point Point::clone() const { return *this; }

void Point::get_coordinates(Standard_Real &x, Standard_Real &y,
                            Standard_Real &z) const {
  x = point.X();
  y = point.Y();
  z = point.Z();
}

double Point::x() const { return point.X(); }
double Point::y() const { return point.Y(); }
double Point::z() const { return point.Z(); }

// Point2D

Point2D Point2D::create(Standard_Real x, Standard_Real y) {
  return Point2D{gp_Pnt2d(x, y)};
}

Point2D Point2D::clone() const { return *this; }

void Point2D::get_coordinates(Standard_Real &x, Standard_Real &y) const {
  x = point.X();
  y = point.Y();
}

double Point2D::x() const { return point.X(); }
double Point2D::y() const { return point.Y(); }

// Vector

Vector Vector::create(Standard_Real x, Standard_Real y, Standard_Real z) {
  return Vector{gp_Vec(x, y, z)};
}

Vector Vector::clone() const { return *this; }

// Direction

Direction Direction::create(Standard_Real x, Standard_Real y, Standard_Real z) {
  return Direction{gp_Dir(x, y, z)};
}

Direction Direction::clone() const { return *this; }

// Direction2D

Direction2D Direction2D::create(Standard_Real x, Standard_Real y) {
  return Direction2D{gp_Dir2d(x, y)};
}

Direction2D Direction2D::clone() const { return *this; }

// Axis

Axis Axis::create(const Point &origin, const Direction &direction) {
  return Axis{gp_Ax1(origin.point, direction.direction)};
}

Axis Axis::clone() const { return *this; }

// Axis2D

Axis2D Axis2D::create(const Point2D &origin, const Direction2D &direction) {
  return Axis2D{gp_Ax2d(origin.point, direction.direction)};
}

Axis2D Axis2D::clone() const { return *this; }

// PlaneAxis

PlaneAxis PlaneAxis::create(const Point &origin, const Direction &direction) {
  return PlaneAxis{gp_Ax2(origin.point, direction.direction)};
}

PlaneAxis PlaneAxis::clone() const { return *this; }

// SpaceAxis

SpaceAxis SpaceAxis::create(const Point &origin, const Direction &direction) {
  return SpaceAxis{gp_Ax3(origin.point, direction.direction)};
}

SpaceAxis SpaceAxis::clone() const { return *this; }

// TrimmedCurve

TrimmedCurve TrimmedCurve::arc_of_circle(const Point &p1, const Point &p2,
                                         const Point &p3) {
  return TrimmedCurve{GC_MakeArcOfCircle(p1.point, p2.point, p3.point)};
}

TrimmedCurve TrimmedCurve::line(const Point &p1, const Point &p2) {
  return TrimmedCurve{GC_MakeSegment(p1.point, p2.point)};
}

TrimmedCurve TrimmedCurve::clone() const { return *this; }

// TrimmedCurve2D

TrimmedCurve2D TrimmedCurve2D::line(const Point2D &p1, const Point2D &p2) {
  return TrimmedCurve2D{GCE2d_MakeSegment(p1.point, p2.point)};
}

TrimmedCurve2D TrimmedCurve2D::clone() const { return *this; }

// Curve2D
Curve2D Curve2D::from_trimmed_curve2d(const TrimmedCurve2D &curve) {
  return Curve2D{const_cast<TrimmedCurve2D &>(curve).curve};
}

Curve2D Curve2D::clone() const { return *this; }

TrimmedCurve2D Curve2D::trim(Standard_Real u1, Standard_Real u2) const {
  return TrimmedCurve2D{new Geom2d_TrimmedCurve(curve, u1, u2)};
}

// Ellipse2D

Ellipse2D Ellipse2D::create(const Axis2D &axis, Standard_Real major_radius,
                            Standard_Real minor_radius) {
  return Ellipse2D{new Geom2d_Ellipse(axis.axis, major_radius, minor_radius)};
}

Ellipse2D Ellipse2D::clone() const { return *this; }

Point2D Ellipse2D::value(Standard_Real u) const {
  return Point2D{ellipse->Value(u)};
}

Curve2D Ellipse2D::curve() const {
  return Curve2D{const_cast<Ellipse2D &>(*this).ellipse};
}

// Plane

Plane Plane::clone() const { return *this; }

Point Plane::location() const { return Point{plane->Location()}; }

// Surface

Surface Surface::from_cylindrical_surface(const CylindricalSurface &surface) {
  return Surface{const_cast<CylindricalSurface &>(surface).surface};
}

Surface Surface::clone() const { return *this; }

bool Surface::is_plane() const {
  return surface->DynamicType() == STANDARD_TYPE(Geom_Plane);
}

Plane Surface::as_plane() const {
  return Plane{Handle(Geom_Plane)::DownCast(surface)};
}

// Transformation

Transformation Transformation::clone() const { return *this; }

void Transformation::mirror(const Axis &axis) {
  transformation.SetMirror(axis.axis);
}

// CylindricalSurface

CylindricalSurface CylindricalSurface::create(const PlaneAxis &axis,
                                              Standard_Real radius) {
  return CylindricalSurface{new Geom_CylindricalSurface(axis.axis, radius)};
}

CylindricalSurface CylindricalSurface::clone() const { return *this; }

} // namespace occara::geom
