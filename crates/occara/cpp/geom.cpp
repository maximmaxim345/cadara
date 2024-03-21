#include "geom.hpp"

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

Vector::Vector(Standard_Real x, Standard_Real y, Standard_Real z)
    : vector(x, y, z) {}

Direction::Direction(Standard_Real x, Standard_Real y, Standard_Real z)
    : direction(x, y, z) {}

Axis::Axis(const Point &origin, const Direction &direction)
    : axis(origin.point, direction.direction) {}

PlaneAxis::PlaneAxis(const Point &origin, const Direction &direction)
    : axis(origin.point, direction.direction) {}

TrimmedCurve::TrimmedCurve(const Point &p1, const Point &p2, const Point &p3)
    : curve(GC_MakeArcOfCircle(p1.point, p2.point, p3.point)) {}

TrimmedCurve::TrimmedCurve(const Point &p1, const Point &p2)
    : curve(GC_MakeSegment(p1.point, p2.point)) {}

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

} // namespace occara::geom
