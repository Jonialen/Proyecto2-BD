#!/bin/bash
set -e # Salir inmediatamente si un comando falla

# --- Configuración ---
# Variables de entorno para la conexión a la BD (pueden definirse aquí o en un .env)
export POSTGRES_USER=${POSTGRES_USER:-user}
export POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-password}
export POSTGRES_DB=${POSTGRES_DB:-mydatabase}

# Directorio y archivo de resultados
resultados_dir="resultados"
output_csv="$resultados_dir/resultados_consolidados.csv"

# Parámetros de la simulación
usuarios_concurrentes=(5 10 20 30)
# Asegúrate que los niveles coincidan EXACTAMENTE con los esperados por el programa Rust (mayúsculas/minúsculas y espacios)
# y los soportados por PostgreSQL
niveles_aislamiento=("READ COMMITTED" "REPEATABLE READ" "SERIALIZABLE")

app_executable="/app/target/release/app" # Ruta correcta para build de release build de release: app_executable="/app/target/release/app"

# --- Preparación ---
echo "Creando directorio de resultados si no existe..."
mkdir -p "$resultados_dir"

echo "Creando archivo CSV de resultados consolidados..."
# Encabezado del CSV (igual al Cuadro 1 del PDF, sin la columna 'ms')
echo "Usuarios Concurrentes,Nivel de Aislamiento,Reservas Exitosas,Reservas Fallidas,Tiempo Promedio (ms)" > "$output_csv"

echo "Construyendo y levantando servicios Docker (una sola vez)..."
# Usamos --force-recreate para asegurar que db inicie limpio (sin volumen, esto es implícito pero más seguro)
# Usamos --build para reconstruir la app si hubo cambios
# Usamos -d para correr en segundo plano
docker compose up --build --force-recreate -d

# Pequeña espera adicional por si acaso, aunque el healthcheck debería ser suficiente
echo "Esperando que la base de datos esté completamente lista (healthcheck)..."
# Puedes añadir un sleep 5 si notas problemas de timing, pero el healthcheck debería manejarlo.

# --- Ejecución de Pruebas ---
echo "Iniciando ciclo de pruebas..."

for usuarios in "${usuarios_concurrentes[@]}"; do
    for nivel in "${niveles_aislamiento[@]}"; do
        echo "-----------------------------------------------------"
        echo "Prueba: Usuarios=$usuarios, Nivel='$nivel'"
        echo "-----------------------------------------------------"

        # 1. Limpiar la tabla de Reservas ANTES de cada prueba específica
        echo "Limpiando tabla Reservas..."
        docker compose exec -T db psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c "TRUNCATE TABLE Reservas RESTART IDENTITY;"
        echo "Tabla Reservas limpiada."

        # 2. Ejecutar la aplicación DENTRO del contenedor 'app'
        echo "Ejecutando simulación en el contenedor app..."
        raw_output=$(docker compose exec -T app sh -c "$app_executable --usuarios $usuarios --nivel-aislamiento \"$nivel\"")

        # Cambiar aquí: Extraer la línea que CONTIENE el formato CSV esperado
        resultado_linea=$(echo "$raw_output" | grep -E '^[0-9]+,"[^"]+",') # Busca líneas con formato CSV

        echo "Salida del programa Rust (línea CSV extraída): $resultado_linea"
        # Opcional: Ver toda la salida si necesitas depurar
        # echo "Salida completa del programa Rust:"
        # echo "$raw_output"
        # echo "-----------------------------------"

        # 3. Añadir resultado al archivo CSV
        if [[ -n "$resultado_linea" && "$resultado_linea" == *","* ]]; then
            echo "$resultado_linea" >> "$output_csv"
            echo "Resultado añadido a $output_csv"
        else
            echo "¡Advertencia! No se obtuvo una línea CSV válida como resultado para Usuarios=$usuarios, Nivel='$nivel'."
            echo "Salida obtenida:"
            echo "$raw_output"
            # Considera añadir una línea de error al CSV o detener el script
            # echo "$usuarios,\"$nivel\",ERROR,ERROR,ERROR" >> "$output_csv"
        fi

    done
done

# --- Limpieza ---
echo "-----------------------------------------------------"
echo "Todas las pruebas completadas."
echo "Resultados consolidados en: $output_csv"
echo "Deteniendo servicios Docker..."
docker compose down # No usamos -v porque ya no hay volúmenes definidos que necesiten limpieza especial

echo "Script finalizado."

