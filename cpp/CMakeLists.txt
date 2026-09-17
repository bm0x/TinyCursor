cmake_minimum_required(VERSION 3.20)
project(TinyCursorCpp LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(TinyCursor WIN32
    main.cpp
    TinyPhysics.hpp
    CanvasRenderer.hpp
    HighRefreshRateLoop.hpp
    app.manifest
)

target_link_libraries(TinyCursor PRIVATE
    d3d11
    dxgi
    d2d1
    dcomp
    user32
    gdi32
    shell32
)

# Embed Windows Application Manifest with uiAccess=true
set_target_properties(TinyCursor PROPERTIES
    WIN32_EXECUTABLE TRUE
    LINK_FLAGS "/MANIFEST:EMBED /MANIFESTINPUT:\"${CMAKE_CURRENT_SOURCE_DIR}/app.manifest\""
)
