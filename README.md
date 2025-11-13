# SpaceTravel #

## Resumen ##

Este repositorio contiene un renderer por software de una simulación de un sistema planetario creada como trabajo del curso. El objetivo del proyecto es diseñar y renderizar un sistema estelar propio usando su motor de pintura de triángulos.

---

## Primera entrega: Carga de modelos

### Imagen del modelo

![Render del modelo](assets/render.png)
 
 ---

## Segunda entrega: Sistema planetario y shaders

En la etapa actual se añadieron:

- Shaders específicos por tipo de cuerpo celeste (`Star`, `Mercury`, `Venus`, `Rocky` (Tierra), `Mars`, `GasGiant` (Júpiter), `Moon`).
- Animaciones basadas en tiempo (rotación propia, detalles que “respiran”, patrones dinámicos).
- Órbita de la Luna alrededor de la Tierra.
- Anillos de Júpiter generados por geometría procedimental en CPU y rasterizados en el mismo framebuffer.
- Contornos dibujados por detección de aristas entre caras front/back para resaltar la silueta.

### Video de la simulación actual

https://github.com/user-attachments/assets/652ff6d3-b695-4fa8-9c4a-01786a231f37


---
