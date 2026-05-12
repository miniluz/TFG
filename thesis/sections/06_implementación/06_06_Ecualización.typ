#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Ecualización

Para la ecualización, se usa un banco de filtros. Un banco de filtros de longitud $|F|$ aplica $|F|$ filtros de paso
bajo, banda y alto en paralelo a una señal para separarla en componentes, cada uno correspondiendo al rango de
frecuencia de su filtro. Estos componentes se escalan independientemente, aumentando o reduciendo el volumen de cada uno
antes de volverlas a añadir. #footnote[Usando la tabla `DB_LINEAR_AMLITUDE_TABLE.`] Para atenuar un rango de frecuencias
en particular o para aumentar las frecuencias agudas, se puede bajar o subir el volumen a los componentes
correspondientes. Idealmente, si no se escala ningún filtro, el ecualizador no afecta la señal.

Sparklet usa 6 filtros. El primero es de paso bajo, los intermedios son de paso banda y el último de paso alto, para
repartir entre ellos todo el rango de frecuencias. Se usan filtros IIR de Butterworth en DF1 @ref_book_theory_music
@ref_book_understanding_dsp, almacenando los coeficientes en formato Q15. Las frecuencias que corresponden a cada filtro
son:

- $250 "Hz"$ (paso bajo),
- entre $500 div sqrt(2) "Hz"$ y $500 times sqrt(2) "Hz"$ (paso banda),
- entre $1000 div sqrt(2) "Hz"$ y $1000 times sqrt(2) "Hz"$ (paso banda),
- entre $2000 div sqrt(2) "Hz"$ y $2000 times sqrt(2) "Hz"$ (paso banda),
- entre $4000 div sqrt(2) "Hz"$ y $4000 times sqrt(2) "Hz"$ (paso banda),
- y $8000 "Hz"$ (paso alto).

A diferencia un banco de filtros ideal, aplicar cada uno de estos filtros y sumar sus componentes sí afecta la señal. En
particular atenúan las frecuencias bajas y altas hasta $6.1 "dB"$ más que las medias. Sin embargo, es necesario usar
filtros de primer orden por rendimiento, y con experimentación se encontró que éstos parámetros dieron los mejores
resultados (teniendo en cuenta que almacenar los coeficientes de un filtro en un Q15 afecta considerablemente su
respuesta).
