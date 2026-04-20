#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Ecualización

Para la ecualización, se usa un banco de filtros. Un banco de filtros de longitud $|F|$ aplica $|F|$ filtros de paso
bajo, banda y alto en paralelo a una señal para separarla en componentes, cada uno correspondiendo al rango de
frecuencia de su respectivo filtro. Estas componentes se escalan independientemente, por ejemplo reduciendo un rango de
frecuencias en particular o aumentando las componentes agudas, antes de volverlas a añadir. Idealmente, si no se escala
ningún filtro, el componente no afecta la señal.

La aplicación usa 6 filtros. El primero es de paso bajo, los intermedios son de paso banda y el último de paso alto,
para repartir entre ellos todo el rango de frecuencia. Se usan filtros IIR de Butterworth en DF1; los coeficientes se
almacenan en formato Q15. Las frecuencias correspondientes a cada filtro son:

- $250 "Hz"$ (paso bajo),
- entre $500 div sqrt(2) "Hz"$ y $500 times sqrt(2) "Hz"$ (paso banda),
- entre $1000 div sqrt(2) "Hz"$ y $1000 times sqrt(2) "Hz"$ (paso banda),
- entre $2000 div sqrt(2) "Hz"$ y $2000 times sqrt(2) "Hz"$ (paso banda),
- entre $4000 div sqrt(2) "Hz"$ y $4000 times sqrt(2) "Hz"$ (paso banda),
- y $8000 "Hz"$ (paso alto).

Estos filtros al combinarse sí afectan la señal, en particular atenuando las frecuencias bajas y altas hasta $6.1 "dB"$
más que las medias. Sin embargo, es necesario usar filtros de primer orden por rendimiento, y bajo experimentación éstos
parámetros dieron los mejores resultados (teniendo en cuenta que almacenar los coeficientes de un filtro en un Q15
afecta considerablemente su respuesta).
