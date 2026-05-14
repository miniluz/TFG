#import "@preview/deal-us-tfc-template:1.0.0": *
#import "../../utils/requirements.typ": req, req-ids, setup-reqs

#show: setup-reqs

== Ecualización

Para la ecualización que pide le @rf_ecualizador, se separa la señal en componentes correspondientes a ciertos rangos de
frecuencia que se escalan independientemente y vuelven a combinar. #footnote[Usando la tabla
  `DB_LINEAR_AMLITUDE_TABLE`.] Para atenuar un rango de frecuencias en particular o para aumentar las frecuencias
agudas, se puede bajar o subir el volumen a los componentes correspondientes.

#let citation = cite(<ref_book_filter_banks>, form: "prose")

En el caso ideal, la suma de todas las bandas sin aplicar ganancia debería reconstruir la señal original sin distorsión.
Para conseguir esto, se podría usar un árbol de filtros perfectamente reconstructivo, como propone #citation. Sin
embargo, implementarlos sin usar operaciones de coma flotante de forma eficiente no es viable, ya que requiere de
filtros FIR de orden grande.

Para conseguir el @rnf_rendimiento, Sparklet usa 6 filtros de Butterworth aplicados en paralelo para dividir la señal en
bandas con solapamiento, de forma no perfectamente reconstructiva. El primero es de paso bajo, los intermedios son de
paso banda y el último de paso alto, para repartir entre ellos todo el rango de frecuencias, como se puede ver en la
@fig_eq_response. Se usan filtros IIR de Butterworth en DF1 @ref_book_theory_music @ref_book_understanding_dsp,
almacenando los coeficientes en formato Q15.

#figure(
  image("/figures/octave_filter_response_q15.png", width: 90%),
  caption: [Respuesta espectral del banco de filtros, calculada usando un barrido sinusoidal en formato Q15. Se muestra
    la respuesta espectral de cada banda y de la suma de todas las bandas.],
  placement: auto,
)<fig_eq_response>

El objetivo del banco es permitir controlar el tono del sonido en términos generales, permitiendo al músico controlar
los componentes graves, medios y altos del sonido. Al usar filtros de Butterworth de primer orden, cada filtro tiene una
pendiente de $-6 "dB"$, lo que resulta en que disminuir el volumen de una banda no resulte brusco. Cada banda se
organiza aproximadamente en una escala de octavas, con solapamiento entre filtros para suavizar la transición entre
bandas.

- $250 "Hz"$ (paso bajo),
- entre $500 div sqrt(2) "Hz"$ y $500 times sqrt(2) "Hz"$ (paso banda),
- entre $1000 div sqrt(2) "Hz"$ y $1000 times sqrt(2) "Hz"$ (paso banda),
- entre $2000 div sqrt(2) "Hz"$ y $2000 times sqrt(2) "Hz"$ (paso banda),
- entre $4000 div sqrt(2) "Hz"$ y $4000 times sqrt(2) "Hz"$ (paso banda),
- y $8000 "Hz"$ (paso alto).

Esta solución atenúa las frecuencias bajas y altas aproximadamente $6,5 "dB"$ más que las medias, como se puede ver en
la @fig_eq_response. Usar estas frecuencias con filtros Butterworth fue la combinación que consiguió los mejores
resultados por experimentación, teniendo en cuenta que almacenar los coeficientes de un filtro en un Q15 afecta
considerablemente su respuesta.
