#pragma once
#include "GC_MakeArcOfCircle.hxx"
#include "GC_MakeSegment.hxx"
#include "Geom_TrimmedCurve.hxx"
#include "gp_Ax1.hxx"
#include "gp_Ax2.hxx"
#include "gp_Pnt.hxx"

namespace occara::geom {

struct Point {
  gp_Pnt point;

  Point(Standard_Real x, Standard_Real y, Standard_Real z);
};

struct Vector {
  gp_Vec vector;

  Vector(Standard_Real x, Standard_Real y, Standard_Real z);
};

struct Direction {
  gp_Dir direction;

  Direction(Standard_Real x, Standard_Real y, Standard_Real z);
};

struct Axis {
  gp_Ax1 axis;

  Axis(const Point &origin, const Direction &direction);
};

struct Axis2d {
  gp_Ax2 axis;

  Axis2d(const Point &origin, const Direction &direction);
};

struct TrimmedCurve {
  Handle(Geom_TrimmedCurve) curve;

  TrimmedCurve(const Point &p1, const Point &p2, const Point &p3);

  TrimmedCurve(const Point &p1, const Point &p2);
};

struct Transformation {
  gp_Trsf transformation;
  Transformation();

  void mirror(const Axis &axis);
};

} // namespace occara::geom
