#include "geom.hpp"
#include "GCE2d_MakeSegment.hxx"

namespace occara::geom {

Point::Point(Standard_Real x, Standard_Real y, Standard_Real z)
    : point(x, y, z) {}

Point::Point(const gp_Pnt &point) : point(point) {}

void Point::get_coordinates(Standard_Real &x, Standard_Real &y,
                            Standard_Real &z) const {
  x = point.X();
  y = point.Y();
  z = point.Z();
}

double Point::x() const { return point.X(); }
double Point::y() const { return point.Y(); }
double Point::z() const { return point.Z(); }

Point2D::Point2D(Standard_Real x, Standard_Real y) : point(x, y) {}

Point2D::Point2D(const gp_Pnt2d &point) : point(point) {}

void Point2D::get_coordinates(Standard_Real &x, Standard_Real &y) const {
  x = point.X();
  y = point.Y();
}

double Point2D::x() const { return point.X(); }
double Point2D::y() const { return point.Y(); }

Vector::Vector(Standard_Real x, Standard_Real y, Standard_Real z)
    : vector(x, y, z) {}

Direction::Direction(Standard_Real x, Standard_Real y, Standard_Real z)
    : direction(x, y, z) {}

Direction2D::Direction2D(Standard_Real x, Standard_Real y) : direction(x, y) {}

Axis::Axis(const Point &origin, const Direction &direction)
    : axis(origin.point, direction.direction) {}

Axis2D::Axis2D(const Point2D &origin, const Direction2D &direction)
    : axis(origin.point, direction.direction) {}

PlaneAxis::PlaneAxis(const Point &origin, const Direction &direction)
    : axis(origin.point, direction.direction) {}

TrimmedCurve::TrimmedCurve(const Point &p1, const Point &p2, const Point &p3)
    : curve(GC_MakeArcOfCircle(p1.point, p2.point, p3.point)) {}

TrimmedCurve::TrimmedCurve(const Point &p1, const Point &p2)
    : curve(GC_MakeSegment(p1.point, p2.point)) {}

TrimmedCurve2D::TrimmedCurve2D(const Geom2d_TrimmedCurve &curve)
    : curve(new Geom2d_TrimmedCurve(curve)) {}

TrimmedCurve2D::TrimmedCurve2D(const Point2D &p1, const Point2D &p2)
    : curve(GCE2d_MakeSegment(p1.point, p2.point)) {}

Ellipse2D::Ellipse2D(const Axis2D &axis, Standard_Real major_radius,
                     Standard_Real minor_radius)
    : ellipse(new Geom2d_Ellipse(axis.axis, major_radius, minor_radius)) {}

TrimmedCurve2D Ellipse2D::trim(Standard_Real u1, Standard_Real u2) const {
  return TrimmedCurve2D(Geom2d_TrimmedCurve(ellipse, u1, u2));
}

Point2D Ellipse2D::value(Standard_Real u) const {
  return Point2D(ellipse->Value(u));
}

Point Plane::location() const { return Point(plane->Location()); }

bool Surface::is_plane() const {
  return surface->DynamicType() == STANDARD_TYPE(Geom_Plane);
}

Plane Surface::as_plane() const {
  return Plane(Handle(Geom_Plane)::DownCast(surface));
}

Transformation::Transformation() : transformation() {}

void Transformation::mirror(const Axis &axis) {
  transformation.SetMirror(axis.axis);
}

CylindricalSurface::CylindricalSurface(const PlaneAxis &axis,
                                       Standard_Real radius)
    : surface(new Geom_CylindricalSurface(axis.axis, radius)) {}

} // namespace occara::geom
