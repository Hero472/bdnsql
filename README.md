
# Base de datos no sql
 
Este es un pequeño proyecto para trabajar con una base de datos NoSQL usando MongoDB y Rust.

## Instalación

Sigue los pasos a continuación para instalar y ejecutar el proyecto en tu máquina local:

### Prerrequisitos:

1. **Instalar Rust**: Descarga e instala Rust desde la [página oficial de Rust](https://www.rust-lang.org/).

    Una vez instalado, verifica la instalación ejecutando:

   ```bash
   rustc --version
   ```

2. **Instalar MongoDB**: Instala MongoDB localmente siguiendo las instrucciones en la [página oficial de MongoDB](https://www.mongodb.com/try/download/community).

3. **Cargo**: Asegúrate de tener `cargo` instalado con Rust. Puedes verificarlo ejecutando:

   ```bash
   cargo --version
   ```

### Clonar el Repositorio

Clona el repositorio del proyecto desde GitHub:

```bash
git clone https://github.com/Hero472/bdnsql.git
cd bdnsql
```

Crea un archivo `.env` en la raíz del proyecto con la URL de tu base de datos MongoDB:

```bash
MONGODB_URI=mongodb://localhost:27017/
```

### Instalar Dependencias

Instala las dependencias del proyecto ejecutando el siguiente comando:

```bash
cargo build
```

### Ejecutar el Proyecto

Una vez que las dependencias estén instaladas, puedes ejecutar el proyecto con:

```bash
cargo run
```

Para poblar la base de datos, puedes usar el siguiente comando:

```bash
cargo run -- --populate
```

### Verificar la Conexión

Si todo ha sido configurado correctamente, deberías ver un mensaje en la consola confirmando la conexión exitosa a MongoDB:

```bash
Connected to MongoDB!
```

## Rutas de API

A continuación se presentan las rutas disponibles en la API:

- **POST** `/units` - Crea una nueva unidad.

  #### Cuerpo de la solicitud (Request Body):
  La solicitud debe enviar un JSON con los siguientes campos:

  ```json
  {
    "course_id": "ObjectId (opcional)",  // El ID del curso al que pertenece la unidad, puede ser nulo.
    "name": "string",                    // El nombre de la unidad.
    "order": "number"                    // El orden de la unidad (entero).
  }
  ```

- **GET** `/units/{course_id}` - Obtiene las unidades asociadas a un curso específico.

  #### Parámetro de la ruta (URL Path Parameter):
  - `course_id`: El ID del curso del que se desea obtener las unidades.

  #### Respuesta (Response Body):
  La respuesta será una lista de unidades asociadas al curso con el siguiente formato:

  ```json
  [
    {
      "unit_id": "ObjectId",               // El ID de la unidad.
      "name": "string",                    // El nombre de la unidad.
      "order": "number"                    // El orden de la unidad (entero).
    }
  ]
  ```


### Cursos

- **POST** `/courses` - Crea un nuevo curso.

  #### Cuerpo de la solicitud (Request Body):
  La solicitud debe enviar un JSON con los siguientes campos:

  ```json
  {
    "name": "string",                      // El nombre del curso.
    "description": "string",               // La descripción del curso.
    "rating": "number (opcional)",         // La calificación del curso (entre 0.0 y 5.0).
    "image": "string",                     // URL de la imagen del curso.
    "image_banner": "string"               // URL del banner del curso.
  }
  ```

- **POST** `/courses` - Crea un nuevo curso completo.

  #### Cuerpo de la solicitud (Request Body):
  La solicitud debe enviar un JSON con los siguientes campos:

  ```json
  {
    "name": "string",                      // El nombre del curso.
    "description": "string",               // La descripción del curso.
    "image": "string",                     // URL de la imagen del curso.
    "image_banner": "string",               // URL del banner del curso.
    "units": [                             // Lista de unidades del curso.
      {
        "name": "string",                  // El nombre de la unidad.
        "order": 0,                        // El orden de la unidad.
        "classes": [                       // Lista de clases dentro de la unidad.
          {
            "name": "string",              // El nombre de la clase.
            "description": "string",       // La descripción de la clase.
            "video": "string",              // URL del video de la clase.
            "tutor": "string",              // Nombre del tutor de la clase.
            "order": 0,                    // El orden de la clase.
            "support_material": ["string"]  // Lista de materiales de soporte.
          }
        ]
      }
    ]
  }

- **GET** `/courses` - Obtiene la lista de cursos disponibles.

  #### Respuesta (Response Body):
  La respuesta será una lista de cursos con un resumen de cada uno:

  ```json
  [
    {
      "name": "string",                      // El nombre del curso.
      "description": "string",               // La descripción del curso.
      "image_banner": "string",              // URL del banner del curso.
      "rating": "number (opcional)"          // La calificación del curso (si existe).
    }
  ]
  ```

- **GET** `/courses/{course_id}` - Obtiene los detalles de un curso específico.

  #### Parámetro de la ruta (URL Path Parameter):
  - `course_id`: El ID del curso del que se desean obtener los detalles.

  #### Respuesta (Response Body):
  La respuesta será un JSON con los detalles completos del curso:

  ```json
  {
    "name": "string",                      // El nombre del curso.
    "description": "string",               // La descripción del curso.
    "image": "string",                     // URL de la imagen del curso.
    "image_banner": "string",              // URL del banner del curso.
    "units": [                             // Lista de unidades asociadas al curso.
      {
        "unit_id": "ObjectId",             // El ID de la unidad.
        "name": "string",                  // El nombre de la unidad.
        "order": "number"                  // El orden de la unidad.
      }
    ]
  }
  ```

- **GET** `/courses/comments/{course_id}` - Obtiene los comentarios de un curso específico.

  #### Parámetro de la ruta (URL Path Parameter):
  - `course_id`: El ID del curso del que se desean obtener los comentarios.

  #### Respuesta (Response Body):
  La respuesta será un JSON con los detalles del curso y una lista de comentarios asociados:

  ```json
  {
    "course": {
      "_id": "ObjectId",                   // El ID del curso.
      "name": "string",                    // El nombre del curso.
      "description": "string",             // La descripción del curso.
      "rating": "number (opcional)",       // La calificación del curso (si existe).
      "image": "string",                   // URL de la imagen del curso.
      "image_banner": "string"             // URL del banner del curso.
    },
    "comments": [                          // Lista de comentarios asociados.
      {
        "author": "string",                // El autor del comentario.
        "date": "string (ISO 8601)",       // Fecha del comentario.
        "title": "string",                 // Título del comentario.
        "detail": "string",                // Detalle del comentario.
        "likes": "number",                 // Número de "me gusta" del comentario.
        "dislikes": "number"               // Número de "no me gusta" del comentario.
      }
    ]
  }
  ```


### Comentarios

- **POST** `/comments` - Crea un nuevo comentario.

  #### Cuerpo de la solicitud (Request Body):
  La solicitud debe enviar un JSON con los siguientes campos:

  ```json
  {
    "author": "string",                     // El nombre del autor del comentario.
    "title": "string",                      // El título del comentario.
    "detail": "string",                     // El detalle o contenido del comentario.
    "reference_id": "ObjectId (opcional)",  // El ID de la referencia asociada al comentario (puede ser nulo).
    "reference_type": "TypeComment"         // El tipo de referencia asociada al comentario (Enum: Class, Course).
  }
  ```


### Clases

- **POST** `/classes` - Crea una nueva clase.

  #### Cuerpo de la solicitud (Request Body):
  La solicitud debe enviar un JSON con los siguientes campos:

  ```json
  {
    "unit_id": "ObjectId (opcional)",        // El ID de la unidad a la que pertenece la clase, puede ser nulo.
    "name": "string",                        // El nombre de la clase.
    "description": "string",                 // La descripción de la clase.
    "order": "number",                       // El orden de la clase en la unidad (entero).
    "video": "string",                       // URL del video de la clase.
    "tutor": "string",                       // Nombre del tutor de la clase.
    "support_material": ["string"]           // Lista de URLs del material de apoyo de la clase.
  }
  ```


- **GET** `/classes/unit/{unit_id}` - Obtiene las clases asociadas a una unidad específica.

  #### Parámetro de la ruta (URL Path Parameter):
  - `unit_id`: El ID de la unidad de la que se desean obtener las clases.

  #### Respuesta (Response Body):
  La respuesta será una lista de clases asociadas a la unidad especificada, con el siguiente formato:

  ```json
  [
    {
      "name": "string",                        // El nombre de la clase.
      "description": "string",                 // La descripción de la clase.
      "order": "number",                       // El orden de la clase en la unidad (entero).
      "video": "string",                       // URL del video de la clase.
      "tutor": "string",                       // Nombre del tutor de la clase.
      "support_material": ["string"]           // Lista de URLs del material de apoyo de la clase.
    }
  ]
  ```

- **GET** `/classes/comments/{class_id}` - Obtiene los comentarios de una clase específica.

  #### Parámetro de la ruta (URL Path Parameter):
  - `class_id`: El ID de la clase de la que se desean obtener los comentarios.

  #### Respuesta (Response Body):
  La respuesta será una lista de comentarios asociados a la clase, cada uno con el siguiente formato:

  ```json
  [
    {
      "author": "string",                   // El nombre del autor del comentario.
      "date": "ISO 8601 timestamp",         // La fecha de creación del comentario.
      "title": "string",                    // El título del comentario.
      "detail": "string",                   // El detalle o contenido del comentario.
      "likes": "number",                    // El número de 'me gusta' recibidos.
      "dislikes": "number"                  // El número de 'no me gusta' recibidos.
    }
  ]
  ```
