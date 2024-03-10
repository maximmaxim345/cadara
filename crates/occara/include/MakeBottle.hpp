#pragma once
#include <TopoDS_Shape.hxx>
#include <memory>

std::unique_ptr<TopoDS_Shape> MakeBottle(const Standard_Real theWidth,
                                         const Standard_Real theHeight,
                                         const Standard_Real theThickness);
