---
layout: ../../layouts/Legal.astro
title: Política de privacidad
description: Qué datos personales tratan los servicios Cymbra, por qué, con quién se comparten y cuáles son tus derechos.
lang: es
updated: 15/10/2026
alternates:
  fr: /confidentialite
  en: /en/privacy
  es: /es/privacidad
---

La presente política explica qué datos personales tratan los **servicios Cymbra**
(editados por **NEETROF**), por qué, con qué base legal, con quién se comparten, durante
cuánto tiempo se conservan y cuáles son tus derechos. Cubre la **cuenta Cymbra**,
compartida entre los servicios Cymbra; los tratamientos propios de cada producto figuran
en un **anexo** (véanse *Anexo A — Cymbra Music* y *Anexo B — Cymbra Lingua*).

## 1. Responsable del tratamiento

**NEETROF — SASU, 948723887**, 42 IMPASSE DUFERMONT, 59510 HEM, FRANCIA.
Contacto: **gfortin@neetrof.fr**.

## 2. Datos que tratamos

Aplicamos la **minimización**: solo recogemos lo necesario para el funcionamiento de la
cuenta y de la aplicación.

| Dato | Origen | Finalidad |
|---|---|---|
| Dirección de correo electrónico | tú (registro por correo electrónico) o tu proveedor (Google/Apple) | identificador de la cuenta, verificación, restablecimiento de la contraseña |
| Contraseña (huella **argon2**, nunca en claro) | tú (registro por correo electrónico) | autenticación |
| Identificador de inicio de sesión externo (identificador «sub» de Google/Apple) | Google / Apple | inicio de sesión mediante «Iniciar sesión con Google/Apple» |
| Nombre de usuario («handle») y nombre mostrado | tú | identificación en la aplicación |
| Preferencias de la aplicación | tú | recordar tus ajustes |
| Tokens de sesión (refresh tokens) | generados al iniciar sesión | mantener tu sesión iniciada |
| Registros técnicos (dirección IP, marcas de tiempo, errores) | servidor | seguridad, prevención de abusos, buen funcionamiento |
| Estado de la suscripción (plan, origen, fechas de inicio y de fin), **identificadores opacos** de la suscripción en el canal de compra (Apple, Google, Paddle — a través de RevenueCat para el App Store y Google Play), campañas beta a las que te has unido, códigos de acceso utilizados | canal de compra / tú | activar el plan Premium en tus dispositivos, gestionar pruebas y betas |

**No** recogemos datos de localización precisa, no vendemos ningún dato y no utilizamos
publicidad de terceros ni rastreadores publicitarios. **Nunca** recibimos ni almacenamos
tus números de tarjeta, direcciones de facturación ni facturas: los trata exclusivamente
el canal de compra (Apple, Google, Paddle).

**Cifras publicadas en la comunidad Cymbra.** Publicamos en los canales públicos de la
comunidad Cymbra (servidor de Discord) cifras de actividad **agregadas**, calculadas a
partir del uso de los servicios: por ejemplo, el número de partidas jugadas, la precisión
media, las partituras puntuadas y las piezas más tocadas en Cymbra Music, las palabras
leídas, aprendidas y repasadas en Cymbra Lingua, o la proporción de cada método de inicio
de sesión entre las cuentas nuevas. Estas cifras no incluyen **ni nombre, ni nombre de
usuario, ni identificador, ni número de personas**. Mientras la comunidad sea pequeña, una
cifra puede, no obstante, reflejar la actividad de muy pocas personas.

## 3. Bases legales (RGPD, art. 6)

- **Ejecución del contrato**: creación y gestión de tu cuenta, prestación de la
  aplicación.
- **Interés legítimo**: seguridad, prevención del fraude y de los abusos, limitación de
  frecuencia, buen funcionamiento, publicación de cifras de actividad agregadas en los
  canales de la comunidad Cymbra (véase el §2).
- **Consentimiento**: inicio de sesión mediante Google/Apple (tú eliges este modo).

## 4. Encargados del tratamiento y terceros

Compartimos lo estrictamente necesario con proveedores que actúan por cuenta nuestra:

- **OVHcloud** (Francia, UE): alojamiento del servidor y de las copias de seguridad.
- **Brevo** (UE): envío de los correos electrónicos transaccionales (verificación,
  restablecimiento).
- **Google** / **Apple**: únicamente si utilizas su inicio de sesión (verificación de tu
  identidad mediante su token), o si te suscribes a través del App Store / Google Play
  (son los vendedores oficiales: cobran y facturan; nosotros nunca vemos tus datos de
  pago).
- **RevenueCat, Inc.** (Estados Unidos): únicamente si te suscribes a través del App Store
  o de Google Play: verifica la compra ante la tienda y sigue el ciclo de vida de la
  suscripción (renovación, periodo de gracia, cancelación, reembolso) por cuenta nuestra,
  y nos proporciona estadísticas agregadas de suscripciones y de ingresos. Recibe un
  **identificador de cuenta opaco**, información técnica sobre la aplicación y el
  dispositivo, y los hechos de la transacción en la tienda (producto, precio, moneda, país,
  fechas) — nunca tu nombre, tu correo electrónico, tu nombre de usuario ni tus datos de
  pago. Esta transferencia fuera de la UE está amparada por las cláusulas contractuales
  tipo de la Comisión Europea (acuerdo de tratamiento de datos firmado con RevenueCat). Su
  ficha se elimina al eliminar tu cuenta.
- **Paddle** (vendedor oficial, Reino Unido/UE): únicamente si te suscribes desde Linux,
  Windows o el sitio web: cobra, factura y nos notifica el estado de la suscripción
  asociada a un identificador opaco.
- **Google** (API Gemini) y **Anthropic, PBC** (API Claude, Estados Unidos): únicamente si
  utilizas la **Traducción en línea** de Cymbra Lingua (*Anexo B*): reciben la frase que hay
  que traducir, sin tu cuenta ni tu email, la traducen y no la conservan. Esta
  transferencia fuera de la UE está amparada por las cláusulas contractuales tipo de la
  Comisión Europea previstas en sus condiciones de tratamiento de datos.

Tus datos se **alojan en la Unión Europea** (Francia). Aparte de la verificación de las
suscripciones y de la Traducción en línea descritas más arriba (RevenueCat, Google y
Anthropic, bajo cláusulas contractuales tipo), no realizamos ninguna transferencia fuera de
la UE.

## 5. Plazo de conservación

- Datos de la cuenta: se conservan **mientras exista tu cuenta**.
- **Eliminación de la cuenta**: cuando eliminas tu cuenta (véase el §7), tus datos
  personales (correo electrónico, huella de la contraseña, identidad externa, nombre de
  usuario, nombre, sesiones) se **borran**.
- Copias de seguridad: las copias de seguridad cifradas rotan en una ventana deslizante
  (14 días) y después se sobrescriben; un dato eliminado desaparece, por tanto, como muy
  tarde al expirar esa ventana.
- Registros técnicos: 7 días.

## 6. Seguridad

Cifrado en tránsito (**TLS**), contraseñas almacenadas como huella **argon2** (nunca en
claro), copias de seguridad **cifradas** y almacenadas fuera del servidor, acceso al
servidor restringido. Como ninguna medida es infalible, no podemos garantizar una seguridad
absoluta.

## 7. Tus derechos (RGPD)

Tienes derecho de **acceso**, **rectificación**, **supresión**, **limitación**,
**oposición** y **portabilidad**.

- **Supresión (derecho al olvido)**: puedes **eliminar tu cuenta directamente en la
  aplicación** (Ajustes → Eliminar cuenta). La eliminación es irreversible y borra tus
  datos personales. Como tu cuenta es común a todas las apps Cymbra, su eliminación se
  aplica a cada una de ellas (Cymbra Music, Cymbra Lingua).
- **Cymbra Lingua**: también puedes borrar **únicamente** tus datos de Lingua, sin
  eliminar tu cuenta (véase el *Anexo B*).
- Para cualquier otra solicitud, escribe a **privacy@cymbra.app**. También puedes
  presentar una reclamación ante la **CNIL** (www.cnil.fr).

## 8. Menores

Cymbra no está destinada a menores de 12 años; no recogemos sus datos a sabiendas.

## 9. Modificaciones

Podemos actualizar esta política; la fecha «Última actualización» que figura arriba se
modificará en consecuencia. En caso de cambio importante, te informaremos.

## 10. Contacto

**gfortin@neetrof.fr** — NEETROF, 42 IMPASSE DUFERMONT, 59510 HEM, FRANCIA.

---

## Anexo A — Cymbra Music

El servicio **Cymbra Music** te permite subir tus propios contenidos. Por ello, además de
lo indicado en el §2, tratamos:

| Dato | Origen | Finalidad |
|---|---|---|
| Archivos subidos (partituras, sonidos de piano / *soundfonts*) | tú | ofrecer las funciones de lectura y de práctica |
| Metadatos asociados (nombre del archivo, declaración de origen, marca de tiempo) | tú | gestión y trazabilidad de tus contenidos |

- **Base legal**: ejecución del contrato (prestación de la funcionalidad).
- **Conservación**: mientras conserves el contenido; la **eliminación** retira el archivo
  y su registro (véanse las Condiciones de uso, *Anexo A — Cymbra Music*).
- Estos archivos se **alojan en la Unión Europea** (Francia), como el resto de tus datos.

---

## Anexo B — Cymbra Lingua

**Cymbra Lingua** (extensión de navegador y app de Safari) resalta, en las páginas que
lees, las palabras que todavía no conoces en los idiomas que estudias. Además de lo indicado
en el §2:

**Lo que se queda en tu dispositivo.** El texto de las páginas se analiza **en tu
dispositivo** y nunca se envía, salvo la frase que envías tú con la **Traducción en línea** y
la traducción de una palabra que decides guardar en su tarjeta (más abajo). También se quedan en el dispositivo: el detalle de tus
lecturas (qué palabras has encontrado, en qué páginas), la **dirección de la página** en la
que añades una palabra a tu mazo y, si no has iniciado sesión, el conjunto de tus datos de
Lingua. También se queda en el dispositivo, sin sincronizarse ni guardarse en el archivo de copia
de seguridad, lo que recuerda la petición de valoración: el número de sesiones de repaso que has
terminado, el número de días en que terminaste alguna, la fecha de la última, el número de veces
que se te mostró la petición y tu respuesta — y, en la app Cymbra Lingua, el número de sesiones
terminadas en la app y la versión de la app para la que se pidió la valoración; la página de la
tienda solo se abre si decides valorar Cymbra Lingua.

**Tus idiomas.** Tus ajustes de idiomas se quedan en el dispositivo: los idiomas que estudias y
tu lengua materna no se guardan como tales. Cada estado, nivel, tarjeta y día de estadísticas
sincronizado se guarda bajo el idioma estudiado al que pertenece, y tu lengua materna llega a
Cymbra como idioma de la traducción de cada tarjeta y con tus estadísticas de cada día; el
idioma de la interfaz de Lingua, o el de tu navegador, se transmite a tu cuenta Cymbra como
idioma de la cuenta (véase más abajo).

**Traducción ampliada.** Este ajuste, desactivado por defecto y propio de cada dispositivo
(no se sincroniza), traduce en tu dispositivo la frase en la que se encuentra tu selección.
Cuando lo activas, la extensión descarga **una sola vez**, desde `models.cymbra.app`, el
modelo o los modelos de Firefox Translations, publicados por Mozilla bajo la licencia MPL 2.0,
que necesitan tus pares de idiomas —un modelo (unos 26 MB) cuando la traducción es directa,
dos (unos 52 MB) cuando pasa por el inglés—; los modelos de un par añadido más tarde solo se
descargan cuando lo pides. A partir de entonces la traducción se hace **en tu dispositivo**,
y el texto de las páginas sigue sin enviarse nunca. Estas descargas no contienen ni texto de
las páginas, ni datos de Lingua, ni token de cuenta, ni identificador de instalación, y
Cymbra no las asocia a ninguna cuenta ni a ninguna instalación. Como con cualquier página
web, el proveedor que aloja estos archivos (Cloudflare) ve la dirección IP del dispositivo
que los solicita. Desactivar el ajuste elimina los modelos del dispositivo. La app Cymbra
Lingua guarda su propia copia de los modelos: solo la descarga, desde la misma dirección y en
las mismas condiciones, cuando la aceptas en la app, si el ajuste está activado en la extensión
de Safari del mismo dispositivo, y la elimina cuando se desactiva.

**Traducción en línea.** Este ajuste, desactivado por defecto y propio de cada dispositivo
(no se sincroniza), solo se ofrece a las cuentas que tienen acceso a él. Cuando está activado,
aparece un botón « Traducción en línea » en la traducción de una selección de varias palabras,
y en una tarjeta en repaso. **No se envía nada mientras no pulses ese botón.** Cuando lo pulsas,
la frase que contiene tu selección —o la frase de la tarjeta repasada, con la palabra localizada
por su posición—, la posición de la selección y el par de idiomas se envían a Cymbra, con tu token de
cuenta, y Cymbra los envía a **Google** (API Gemini) para traducirlos, o a **Anthropic** (API
Claude) cuando Google no responde. Ni Cymbra ni estos proveedores conservan la frase ni su
traducción, y estos proveedores no la usan para entrenar sus modelos. Para limitar el uso y
conocer su coste, Cymbra cuenta, para tu cuenta, las traducciones en línea del día (un contador
que se borra al cabo de un día) y guarda totales diarios por proveedor y par de idiomas, sin
cuenta ni texto. En la app Cymbra Lingua, la Traducción en línea te pide iniciar sesión **en la
app** (con Apple, Google o tu correo y tu contraseña): la app abre entonces su propia sesión de tu cuenta, distinta de la
de la extensión, que puedes cerrar desde la app.

**Traducción guardada.** Una traducción automática (en el dispositivo o en línea) solo se
conserva si, en el repaso, decides guardar la traducción de la palabra como traducción de su
tarjeta: solo se guarda esa palabra traducida, en la tarjeta, y se sincroniza con ella; la frase
traducida nunca. Una traducción que no guardas no se conserva en ningún sitio.

**Lectura en voz alta.** La lectura en voz alta utiliza una voz del idioma leído, instalada
en tu dispositivo: el texto leído no sale de él. Si no hay ninguna voz de ese idioma
instalada, puedes activar como alternativa las voces en línea de tu navegador (ajuste
desactivado por defecto, propio de cada dispositivo): el texto leído lo envía entonces **tu
navegador** al proveedor de esas voces (Google, en el caso de Chrome), según las condiciones
de ese proveedor. Cymbra no recibe ese texto. En cuanto haya una voz de ese idioma instalada
en el dispositivo, se utiliza en su lugar.

**Recuentos anónimos.** Para saber si Lingua es realmente útil a quienes la instalan, la
extensión puede enviar a Cymbra seis recuentos, cada uno una vez por instalación: Lingua está
**instalada**; está **lista** (un primer nivel elegido); se **analiza una primera página** en un
idioma que estudias; vuelves a usarla **al día siguiente** de instalarla; de nuevo **entre el
día 7 y el 13**; de nuevo **entre el día 30 y el 59**. Cada envío solo contiene tres datos: la **semana** de la
instalación, la **familia del navegador** (Chromium, Firefox o Safari) y la **versión** de la
extensión (por ejemplo, 1.10).

- **Ningún identificador** de ningún tipo —ni identificador de instalación o de dispositivo, ni
  cuenta, aunque hayas iniciado sesión— y **nada de lo que lees**: ni dirección, ni sitio, ni
  texto de la página, ni palabra, ni idioma estudiado, ni nivel.
- **Ninguna dirección IP conservada.** Como excepción a los registros técnicos del §2 y del §5,
  Cymbra no guarda la dirección de red de estos envíos ni en su base de datos, ni en sus
  registros, ni en los del servidor que le transmite las solicitudes (su « proxy inverso »).
  Solo sirve para rechazar los envíos masivos, únicamente en memoria y de forma ilegible, y se
  olvida al cabo de una hora.
- **Enviados la semana siguiente.** Cada recuento sale de tu dispositivo solo a partir del lunes
  que sigue a la semana en que ocurrió, al siguiente inicio o uso de la extensión: el momento en
  que llega a Cymbra no dice el día. Si la extensión se detiene justo cuando Cymbra recibe un
  envío, ese envío vuelve a salir más tarde y cuenta dos veces: los totales solo pueden quedar un
  poco altos, nunca vinculados a ti.
- **En tu dispositivo**, el día de la instalación se guarda para saber cuándo vuelves —hasta el
  final de la semana siguiente a la del día 59, **73 días como máximo**— y después se olvida; solo
  sale de él su semana. En Cymbra, los recuentos son totales semanales, sin rastro de cada envío,
  y **se borran a los 25 meses**.
- **Instalaciones anteriores a los recuentos.** Una instalación hecha antes de la versión de
  Lingua que introdujo los recuentos nunca se cuenta, nunca se le pregunta y no abre ninguna
  página de despedida.
- **Cómo pregunta cada navegador.** Chrome y los demás navegadores Chromium (Edge…): no se envía
  nada hasta que pulsas « Sí, contar ». Firefox: decide el propio consentimiento de Firefox a los
  « Datos técnicos y de interacción », que se ofrece al instalar la extensión (Firefox 140 o
  posterior de escritorio, 142 o posterior en Android; las versiones anteriores no cuentan nada).
  Safari: activados por defecto, una vez mostrada la frase que los presenta.
- **Desactivarlos.** El interruptor « Recuentos anónimos », en Ajustes → Datos, los desactiva en
  cualquier momento, en todos los navegadores (en Firefox, actúa sobre ese mismo
  consentimiento). Al desactivarlos se olvida lo que aún no se había enviado.
- **Base legal**: en Chrome y los demás navegadores Chromium, tu **consentimiento**; en los
  demás, el **interés legítimo** de Cymbra en medir la audiencia de Lingua. El interruptor es la
  forma de oponerte, o de retirar tu consentimiento, en cualquier momento.

**La página de despedida.** En Chrome y Firefox, mientras los recuentos están activados,
desinstalar Lingua abre una página de cymbra.app con una sola pregunta —por qué te vas— y seis
respuestas fijas, sin campo libre. Abrirla es una visita normal al sitio: como en cualquiera de
sus páginas, el proveedor que lo aloja (Cloudflare) recibe la dirección IP de tu dispositivo y la
dirección de la página, con sus dos parámetros (la familia del navegador y la versión); Cymbra no
añade nada a esa visita ni recibe nada de ella. Solo la respuesta que pulsas se envía a Cymbra y
se cuenta, de la misma forma anónima: la semana en que llega, la familia del navegador y la
versión de la extensión, los únicos datos que lleva la dirección de la página (sin JavaScript, la
página no envía ninguno de los dos, y se cuentan como desconocidos). Con los recuentos
desactivados, no se abre ninguna página.

**Lo que se sincroniza si has iniciado sesión** con tu cuenta Cymbra:

| Dato | Origen | Finalidad |
|---|---|---|
| Estado de cada palabra (conocida, en aprendizaje, ignorada) y nivel declarado, para cada idioma que estudias | tú | recuperar tu progreso en tus dispositivos |
| Mazo de repaso, para cada idioma que estudias: la palabra, la frase en la que la encontraste, su traducción —la del diccionario, o la traducción automática de la palabra que decidiste guardar, nunca una frase traducida—, el idioma de esa traducción y su estado de repaso —**sin** la dirección de la página | tú | repasar en todos tus dispositivos |
| Estadísticas por día y por idioma que estudias (palabras aprendidas, repasos, número de palabras leídas y, entre ellas, de palabras nuevas), con tu lengua materna de ese día (la de las traducciones) | calculadas en el dispositivo | mostrar tus estadísticas; cifras de uso **agregadas** para hacer funcionar el servicio y, sin nombre ni número de personas, publicadas en los canales de la comunidad Cymbra (véase el §2) |
| Identificador de instalación aleatorio | generado por la extensión, y por la app Cymbra Lingua para sí misma | distinguir dos dispositivos durante la sincronización |

La extensión sincroniza estos datos. Si **también inicias sesión en la app Cymbra Lingua**, con la
cuenta que usa la extensión de Safari, la propia app sincroniza los estados y las tarjetas de los
repasos hechos en la app, con su propia sesión y su propio identificador de instalación, con el
mismo contenido (sin dirección de página); las estadísticas del día siguen pasando por la
extensión. No se recoge nada más.

- **Base legal**: ejecución del contrato (prestación de la sincronización); interés
  legítimo para la publicación de cifras agregadas (véase el §2).
- **Conservación**: mientras exista tu cuenta, o hasta que borres tus datos de Lingua.
- **Borrar tus datos de Lingua sin eliminar tu cuenta**: en la extensión, en la página de la
  cuenta, la opción que borra tus datos de Lingua. El borrado se aplica al servidor y a
  todos tus dispositivos (cada uno se borra en su próxima sincronización); tu cuenta Cymbra
  y tus otras apps Cymbra no se ven afectadas.
- La **eliminación de tu cuenta** (§7) borra también todos tus datos de Lingua.
- Estos datos se **alojan en la Unión Europea** (Francia), como el resto de tus datos.

**Tu cuenta Cymbra.** Cuando creas tu cuenta, pides un código o una nueva contraseña, o añades
una contraseña desde Lingua, la extensión indica a Cymbra un idioma —el de la interfaz de
Lingua o el de tu navegador— que tu cuenta guarda como su idioma: Cymbra te escribe en él
(verificación, restablecimiento de la contraseña) y tus otras apps Cymbra pueden adoptarlo.

**Política de Datos de Usuario de Chrome Web Store.** « The use of information received from
Google APIs will adhere to the Chrome Web Store User Data Policy, including the Limited Use
requirements. » Es decir: el uso de la información recibida de las API de Google se ajustará a
la Política de Datos de Usuario de Chrome Web Store (*Chrome Web Store User Data Policy*),
incluidos los requisitos de Uso Limitado (*Limited Use*). Para Cymbra Lingua, esta información
es: el texto de las páginas que lees, analizado en tu dispositivo; tu cuenta Cymbra (dirección
de correo electrónico, identificador de inicio de sesión, sesiones, idioma de la cuenta); lo
que se sincroniza si has iniciado sesión (estados y niveles de las palabras, mazo de repaso,
estadísticas diarias, identificador de instalación); y la frase enviada con la **Traducción en
línea**, solo cuando pulsas su botón. Cymbra solo usa estos datos para los fines descritos en
esta política, solo los comparte con los encargados del tratamiento citados en el §4 y para
esos fines, no los vende y no los usa para publicidad. El equipo de Cymbra solo ve tus datos de
Lingua como totales, sin cuenta, salvo para ayudarte a petición tuya, por seguridad o para
cumplir la ley.
