#pragma once
#include "shape.hpp"
#include <TopoDS_Shape.hxx>
#include <memory>

occara::shape::Shape MakeBottle(const Standard_Real theWidth,
                                const Standard_Real theHeight,
                                const Standard_Real theThickness);
