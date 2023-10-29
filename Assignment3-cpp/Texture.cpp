//
// Created by LEI XU on 4/27/19.
//

#include "Texture.hpp"

Eigen::Vector3f Texture::getColorBilinear(float u, float v) {
    auto u_img = u * width;
    auto v_img = (1 - v) * height;

    int u_min = (int)floor(u_img);
    int v_min = (int)floor(v_img);
    int u_max = (int)ceil(u_img);
    int v_max = (int)ceil(v_img);

    float s = u_img - u_min;
    float t = v_img - v_min;

    auto color1 = getColor(u_min, v_min);
    auto color2 = getColor(u_max, v_min);
    auto color3 = getColor(u_min, v_max);
    auto color4 = getColor(u_max, v_max);

    auto color12 = (1 - s) * color1 + s * color2;
    auto color34 = (1 - s) * color3 + s * color4;

    auto color = (1 - t) * color12 + t * color34;

    return color;
}

