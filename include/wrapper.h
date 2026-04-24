#pragma once

// Rename main function so it doesn't conflict when it's
// exported via wrap_static_fns build of rust-bindgen
#define main gztool_main

#include <gztool.c>
