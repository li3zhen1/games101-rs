//
// Created by Göksu Güvendiren on 2019-05-14.
//

#include "Scene.hpp"


void Scene::buildBVH() {
    printf(" - Generating BVH...\n\n");
    this->bvh = new BVHAccel(objects, 1, BVHAccel::SplitMethod::NAIVE);
}

Intersection Scene::intersect(const Ray &ray) const {
    return this->bvh->Intersect(ray);
}

void Scene::sampleLight(Intersection &pos, float &pdf) const {
    float emit_area_sum = 0;
    for (uint32_t k = 0; k < objects.size(); ++k) {
        if (objects[k]->hasEmit()) {
            emit_area_sum += objects[k]->getArea();
        }
    }
    float p = get_random_float() * emit_area_sum;
    emit_area_sum = 0;
    for (uint32_t k = 0; k < objects.size(); ++k) {
        if (objects[k]->hasEmit()) {
            emit_area_sum += objects[k]->getArea();
            if (p <= emit_area_sum) {
                objects[k]->Sample(pos, pdf);
                break;
            }
        }
    }
}

bool Scene::trace(
    const Ray &ray,
    const std::vector<Object *> &objects,
    float &tNear, uint32_t &index, Object **interect) {
    *interect = nullptr;
    for (uint32_t k = 0; k < objects.size(); ++k) {
        float tNearK = kInfinity;
        uint32_t indexK;
        Vector2f uvK;
        if (objects[k]->intersect(ray, tNearK, indexK) && tNearK < tNear) {
            *interect = objects[k];
            tNear = tNearK;
            index = indexK;
        }
    }


    return (*interect != nullptr);
}


constexpr float epsilon = 0.0002f;

Vector3f Scene::shadeLoDir(Intersection &inter, Vector3f wo) const {
    float lightPdf;
    Vector3f result;
    Intersection hitLight;
    sampleLight(hitLight, lightPdf);
    Vector3f obj2Light = hitLight.coords - inter.coords;
    Vector3f obj2LightDir = obj2Light.normalized();

    auto t = intersect(Ray(inter.coords, obj2LightDir));
    if (t.distance - obj2Light.norm() > -epsilon) {
        Vector3f f_r = inter.m->eval(obj2LightDir, wo, inter.normal);
        float r2 = dotProduct(obj2Light, obj2Light);
        float cosA = std::max(.0f, dotProduct(inter.normal, obj2LightDir));
        float cosB = std::max(.0f, dotProduct(hitLight.normal, -obj2LightDir));
        result = hitLight.emit * f_r * cosA * cosB / r2 / lightPdf;
    }
    return result;
}

Vector3f Scene::shadeLoIndir(Intersection &inter, Vector3f wo) const {
    Vector3f result;
    if (get_random_float() < RussianRoulette) {
        Vector3f dir2NextObj = inter.m->sample(wo, inter.normal).normalized();
        float pdf = inter.m->pdf(wo, dir2NextObj, inter.normal);
        if (pdf > epsilon) {
            Intersection nextObj = intersect(Ray(inter.coords, dir2NextObj));
            if (nextObj.happened && !nextObj.m->hasEmission()) {
                Vector3f f_r = inter.m->eval(dir2NextObj, wo, inter.normal); //BRDF
                float cos = std::max(.0f, dotProduct(dir2NextObj, inter.normal));
                result = shade(nextObj, -dir2NextObj) * f_r * cos / pdf / RussianRoulette;
            }
        }
    }
    return result;
}

Vector3f Scene::shade(Intersection &inter, Vector3f r) const {
    if (inter.m->hasEmission()) {
        return inter.m->getEmission();
    }
    return shadeLoDir(inter, r) + shadeLoIndir(inter, r);
}

// Implementation of Path Tracing
Vector3f Scene::castRay(const Ray &ray, int depth) const {

    auto inter = intersect(ray);
    if (!inter.happened) return {};
    return shade(inter, -ray.direction);
}