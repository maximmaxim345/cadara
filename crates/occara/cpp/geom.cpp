#include "geom.hpp"

namespace occara::geom {

Point::Point(Standard_Real x, Standard_Real y, Standard_Real z)
    : point(x, y, z) {}

Vector::Vector(Standard_Real x, Standard_Real y, Standard_Real z)
    : vector(x, y, z) {}

Direction::Direction(Standard_Real x, Standard_Real y, Standard_Real z)
    : direction(x, y, z) {}

Axis::Axis(const Point &origin, const Direction &direction)
    : axis(origin.point, direction.direction) {}

Axis2d::Axis2d(const Point &origin, const Direction &direction)
    : axis(origin.point, direction.direction) {}

TrimmedCurve::TrimmedCurve(const Point &p1, const Point &p2, const Point &p3)
    : curve(GC_MakeArcOfCircle(p1.point, p2.point, p3.point)) {}

TrimmedCurve::TrimmedCurve(const Point &p1, const Point &p2)
    : curve(GC_MakeSegment(p1.point, p2.point)) {}

Transformation::Transformation() : transformation() {}

void Transformation::mirror(const Axis &axis) {
  transformation.SetMirror(axis.axis);
}

} // namespace occara::geom
