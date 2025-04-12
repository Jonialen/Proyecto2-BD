# Proyecto2-BD: Simulador de Reservas con PostgreSQL

Este proyecto es un simulador de reservas de asientos para eventos que permite evaluar el comportamiento de diferentes niveles de aislamiento en PostgreSQL bajo condiciones de alta concurrencia.

## Descripción

El sistema simula múltiples usuarios intentando reservar asientos simultáneamente en una base de datos PostgreSQL, midiendo:
- Número de reservas exitosas
- Número de reservas fallidas
- Tiempo promedio de transacciones exitosas

## Características principales

- Soporte para tres niveles de aislamiento de PostgreSQL:
  - READ COMMITTED
  - REPEATABLE READ
  - SERIALIZABLE
- Configuración de número de usuarios concurrentes
- Generación de reportes en formato CSV
- Sistema de logging detallado
- Entorno Dockerizado para fácil despliegue

## Requisitos

- Docker
- Docker Compose
- Rust (si se desea compilar localmente)

## Configuración

1. Clonar el repositorio
2. Configurar variables de entorno en `.env` (opcional):

POSTGRES_USER=user
POSTGRES_PASSWORD=password
POSTGRES_DB=mydatabase

## Uso

Ejecutar el script de pruebas:

./test.sh

Este script:
1. Construye y levanta los contenedores Docker
2. Ejecuta las pruebas con diferentes configuraciones
3. Genera un archivo CSV con los resultados
4. Limpia los recursos

## Estructura del proyecto

- `app/`: Aplicación Rust que realiza las simulaciones
- `01_ddl.sql`: Esquema de la base de datos
- `02_data.sql`: Datos iniciales
- `docker-compose.yml`: Configuración de Docker
- `test.sh`: Script de pruebas automatizadas

## Resultados

Los resultados se guardan en `resultados/resultados_consolidados.csv` con el siguiente formato:

Usuarios Concurrentes,Nivel de Aislamiento,Reservas Exitosas,Reservas Fallidas,Tiempo Promedio (ms)

## Personalización

Se pueden modificar los parámetros de prueba en `test.sh`:
- `usuarios_concurrentes`: Array con número de usuarios a simular
- `niveles_aislamiento`: Niveles de aislamiento a evaluar

## Licencia

Este proyecto está bajo la licencia MIT.

