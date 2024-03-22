#pragma once
#include "GC_MakeArcOfCircle.hxx"
#include "GC_MakeSegment.hxx"
#include "Geom2d_Ellipse.hxx"
#include "Geom2d_TrimmedCurve.hxx"
#include "Geom_CylindricalSurface.hxx"
#include "Geom_Plane.hxx"
#include "Geom_Surface.hxx"
#include "Geom_TrimmedCurve.hxx"
#include "gp_Ax1.hxx"
#include "gp_Ax2.hxx"
#include "gp_Dir2d.hxx"
#include "gp_Pnt.hxx"
#include "gp_Pnt2d.hxx"

namespace occara::geom {

// Forward declarations
struct Point;
struct Point2D;
struct Vector;
struct Direction;
struct Direction2D;
struct Axis;
struct Axis2D;
struct PlaneAxis;
struct TrimmedCurve;
struct TrimmedCurve2D;
struct Ellipse2D;
struct Plane;
struct Surface;
struct Transformation;
struct CylindricalSurface;

struct Point {
  gp_Pnt point;

  Point(Standard_Real x, Standard_Real y, Standard_Real z);
  Point(const gp_Pnt &point);

  void get_coordinates(Standard_Real &x, Standard_Real &y,
                       Standard_Real &z) const;
  Standard_Real x() const;
  Standard_Real y() const;
  Standard_Real z() const;
};

struct Point2D {
  gp_Pnt2d point;

  Point2D(Standard_Real x, Standard_Real y);
  Point2D(const gp_Pnt2d &point);

  void get_coordinates(Standard_Real &x, Standard_Real &y) const;
  Standard_Real x() const;
  Standard_Real y() const;
};

struct Vector {
  gp_Vec vector;

  Vector(Standard_Real x, Standard_Real y, Standard_Real z);
};

struct Direction {
  gp_Dir direction;

  Direction(Standard_Real x, Standard_Real y, Standard_Real z);
};

struct Direction2D {
  gp_Dir2d direction;

  Direction2D(Standard_Real x, Standard_Real y);
};

struct Axis {
  gp_Ax1 axis;

  Axis(const Point &origin, const Direction &direction);
};

struct Axis2D {
  gp_Ax2d axis;

  Axis2D(const Point2D &origin, const Direction2D &direction);
};

struct PlaneAxis {
  gp_Ax2 axis;

  PlaneAxis(const Point &origin, const Direction &direction);
};

struct TrimmedCurve {
  Handle(Geom_TrimmedCurve) curve;

  TrimmedCurve(const Point &p1, const Point &p2, const Point &p3);
  TrimmedCurve(const Point &p1, const Point &p2);
};

struct TrimmedCurve2D {
  Handle(Geom2d_TrimmedCurve) curve;

  TrimmedCurve2D(const Geom2d_TrimmedCurve &curve);
  TrimmedCurve2D(const Point2D &p1, const Point2D &p2);
};

struct Ellipse2D {
  Handle(Geom2d_Ellipse) ellipse;

  Ellipse2D(const Axis2D &axis, Standard_Real major_radius,
            Standard_Real minor_radius);

  TrimmedCurve2D trim(Standard_Real u1, Standard_Real u2) const;
  Point2D value(Standard_Real u) const;
};

struct Plane {
  Handle(Geom_Plane) plane;

  Point location() const;
};

struct Surface {
  Handle(Geom_Surface) surface;

  bool is_plane() const;
  Plane as_plane() const;
};

struct Transformation {
  gp_Trsf transformation;

  Transformation();

  void mirror(const Axis &axis);
};

struct CylindricalSurface {
  Handle(Geom_CylindricalSurface) surface;

  CylindricalSurface(const PlaneAxis &axis, Standard_Real radius);
};

} // namespace occara::geom
