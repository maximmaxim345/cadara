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

  static Point create(Standard_Real x, Standard_Real y, Standard_Real z);
  Point clone() const;

  void get_coordinates(Standard_Real &x, Standard_Real &y,
                       Standard_Real &z) const;
  Standard_Real x() const;
  Standard_Real y() const;
  Standard_Real z() const;
};

struct Point2D {
  gp_Pnt2d point;

  static Point2D create(Standard_Real x, Standard_Real y);
  Point2D clone() const;

  void get_coordinates(Standard_Real &x, Standard_Real &y) const;
  Standard_Real x() const;
  Standard_Real y() const;
};

struct Vector {
  gp_Vec vector;

  static Vector create(Standard_Real x, Standard_Real y, Standard_Real z);
  Vector clone() const;
};

struct Direction {
  gp_Dir direction;

  static Direction create(Standard_Real x, Standard_Real y, Standard_Real z);
  Direction clone() const;
};

struct Direction2D {
  gp_Dir2d direction;

  static Direction2D create(Standard_Real x, Standard_Real y);
  Direction2D clone() const;
};

struct Axis {
  gp_Ax1 axis;

  static Axis create(const Point &origin, const Direction &direction);
  Axis clone() const;
};

struct Axis2D {
  gp_Ax2d axis;

  static Axis2D create(const Point2D &origin, const Direction2D &direction);
  Axis2D clone() const;
};

struct PlaneAxis {
  gp_Ax2 axis;

  static PlaneAxis create(const Point &origin, const Direction &direction);
  PlaneAxis clone() const;
};

struct SpaceAxis {
  gp_Ax3 axis;

  static SpaceAxis create(const Point &origin, const Direction &direction);
  SpaceAxis clone() const;
};

struct TrimmedCurve {
  Handle(Geom_TrimmedCurve) curve;

  static TrimmedCurve arc_of_circle(const Point &p1, const Point &p2,
                                    const Point &p3);
  static TrimmedCurve line(const Point &p1, const Point &p2);
  TrimmedCurve clone() const;
};

struct TrimmedCurve2D {
  Handle(Geom2d_TrimmedCurve) curve;

  static TrimmedCurve2D line(const Point2D &p1, const Point2D &p2);
  TrimmedCurve2D clone() const;
};

struct Ellipse2D {
  Handle(Geom2d_Ellipse) ellipse;

  static Ellipse2D create(const Axis2D &axis, Standard_Real major_radius,
                          Standard_Real minor_radius);
  Ellipse2D clone() const;

  TrimmedCurve2D trim(Standard_Real u1, Standard_Real u2) const;
  Point2D value(Standard_Real u) const;
};

struct Plane {
  Handle(Geom_Plane) plane;

  Plane clone() const;

  Point location() const;
};

struct Surface {
  Handle(Geom_Surface) surface;

  Surface clone() const;

  bool is_plane() const;
  Plane as_plane() const;
};

struct Transformation {
  gp_Trsf transformation;

  Transformation clone() const;

  void mirror(const Axis &axis);
};

struct CylindricalSurface {
  Handle(Geom_CylindricalSurface) surface;

  static CylindricalSurface create(const PlaneAxis &axis, Standard_Real radius);
  CylindricalSurface clone() const;
};

} // namespace occara::geom
