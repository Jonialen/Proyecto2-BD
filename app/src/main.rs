// Archivo: proyecto2/app/src/main.rs (MODIFICADO)

use chrono::Utc;
use clap::Parser;
use dotenv::dotenv;
use futures::future::join_all;
use rand::Rng; // Ya estaba importado, necesario para gen_range
use sqlx::{Executor, Postgres, Transaction};
use sqlx::postgres::PgPoolOptions;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    usuarios: usize,

    #[arg(short, long)]
    nivel_aislamiento: String,
}

#[derive(Debug, Default)]
struct Resultados {
    exitosas: AtomicUsize,
    fallidas: AtomicUsize,
    tiempos_ms: AtomicUsize, // Acumulador para sumar tiempos
}

async fn simular_reserva(
    pool: Arc<sqlx::Pool<Postgres>>,
    resultados: Arc<Resultados>,
    nivel_aislamiento: &str,
) -> Result<bool, sqlx::Error> {
    let start_time = Instant::now();

    // --- INICIO DEL CAMBIO ---
    // En lugar de un asiento fijo, seleccionamos uno aleatorio entre 1 y 10
    // Usamos ..= para incluir el 10 en el rango.
    let asiento_id: i32 = rand::thread_rng().gen_range(1..=10);
    // --- FIN DEL CAMBIO ---

    let mut tx: Transaction<Postgres> = match pool.begin().await {
        Ok(tx) => tx,
        Err(e) => {
            error!("Error al iniciar transacción: {}", e);
            resultados.fallidas.fetch_add(1, Ordering::SeqCst);
            return Err(e);
        }
    };

    // Creamos el generador DESPUÉS del await (esto ya estaba bien)
    let usuario_id: i32 = rand::thread_rng().gen_range(1..10000); // ID de usuario aleatorio

    // Establecer nivel de aislamiento
    let nivel_query = format!("SET TRANSACTION ISOLATION LEVEL {}", nivel_aislamiento);
    if let Err(e) = tx.execute(sqlx::query(&nivel_query)).await {
        error!("Error al establecer nivel de aislamiento: {}", e);
        resultados.fallidas.fetch_add(1, Ordering::SeqCst);
        let _ = tx.rollback().await; // Intentar rollback
        return Err(e);
    }

    // Intentar insertar la reserva
    let query = "INSERT INTO Reservas (asiento_id, usuario_id, timestamp_reserva) VALUES ($1, $2, $3)";
    let resultado_insert = tx
        .execute(sqlx::query(query).bind(asiento_id).bind(usuario_id).bind(Utc::now()))
        .await;

    match resultado_insert {
        Ok(_) => {
            // Si el INSERT tiene éxito, intentar COMMIT
            match tx.commit().await {
                Ok(_) => {
                    let duration_ms = start_time.elapsed().as_millis();
                    resultados.exitosas.fetch_add(1, Ordering::SeqCst);
                    // Sumamos el tiempo de esta transacción exitosa al total
                    resultados.tiempos_ms.fetch_add(duration_ms as usize, Ordering::SeqCst);
                    // Usamos asiento_id en el log para ver cuál se reservó
                    info!(usuario_id, asiento_id, duration_ms, "Reserva exitosa (COMMIT OK).");
                    Ok(true) // Indica éxito
                }
                Err(e) => {
                    // Error durante COMMIT (raro, pero posible)
                    error!(usuario_id, asiento_id, "Error durante COMMIT: {}", e);
                    resultados.fallidas.fetch_add(1, Ordering::SeqCst);
                    Err(e)
                }
            }
        }
        Err(e) => {
            // Error durante INSERT (incluye duplicate key, deadlock, etc.)
            // Intentar ROLLBACK
            if let Err(rollback_err) = tx.rollback().await {
                error!(usuario_id, asiento_id, "Error en INSERT ({}) y ROLLBACK: {}", e, rollback_err);
            } else {
                // Rollback exitoso tras error de INSERT
                error!(usuario_id, asiento_id, "Error en INSERT, ROLLBACK ejecutado: {}", e);
            }
            resultados.fallidas.fetch_add(1, Ordering::SeqCst);
            Err(e) // Devuelve el error original del INSERT
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuración de tracing/logging (igual que antes)
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO) // Puedes cambiar a DEBUG si necesitas más detalle
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default subscriber failed");

    dotenv().ok(); // Cargar variables de entorno si existe .env

    // Parsear argumentos (igual que antes)
    let args = Args::parse();
    let num_usuarios = args.usuarios;
    let nivel_aislamiento_str = args.nivel_aislamiento.to_uppercase(); // Asegurar mayúsculas

    info!(
        usuarios = num_usuarios,
        nivel = nivel_aislamiento_str,
        "Iniciando simulación."
    );

    // Conexión a la base de datos (igual que antes)
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL debe estar definida en el entorno");

    let pool = Arc::new(
        PgPoolOptions::new()
            .max_connections(50) // Ajusta según necesidad y recursos
            .connect(&database_url)
            .await
            .map_err(|e| {
                error!("Error al conectar al pool de BD: {}", e);
                e // Propagar el error
            })?,
    );

    // Estructura para resultados (igual que antes)
    let resultados = Arc::new(Resultados::default());
    let mut handles = vec![];

    // Lanzar tareas concurrentes (igual que antes)
    for i in 0..num_usuarios {
        let pool_clone = Arc::clone(&pool);
        let resultados_clone = Arc::clone(&resultados);
        let nivel_clone = nivel_aislamiento_str.clone(); // Clonar para mover al task

        let handle = tokio::spawn(async move {
            // Manejar el resultado de simular_reserva (éxito o error SQL)
            match simular_reserva(pool_clone, resultados_clone, &nivel_clone).await {
                Ok(exito) => {
                    if !exito {
                        // Esto no debería ocurrir ahora ya que devolvemos Err en caso de fallo
                        info!("Tarea {} completada con fallo lógico (inesperado).", i);
                    }
                    // El éxito ya se loguea dentro de simular_reserva
                }
                Err(_) => {
                    // El error ya se loguea dentro de simular_reserva
                    info!("Tarea {} completada con error SQL (esperado en caso de conflicto).", i);
                }
            }
        });

        handles.push(handle);
    }

    // Esperar a que todas las tareas terminen (igual que antes)
    join_all(handles).await;

    // Calcular y mostrar resultados (igual que antes)
    let exitosas = resultados.exitosas.load(Ordering::SeqCst);
    let fallidas = resultados.fallidas.load(Ordering::SeqCst);
    let total_tiempo_ms = resultados.tiempos_ms.load(Ordering::SeqCst);
    let promedio_millis = if exitosas > 0 {
        // Calcular promedio solo si hubo éxitos
        (total_tiempo_ms as f64 / exitosas as f64) as u64
    } else {
        0 // Evitar división por cero
    };

    // Imprimir la línea CSV (igual que antes)
    println!(
        "{},\"{}\",{},{},{}",
        num_usuarios, // Usar el num_usuarios original pasado como argumento
        args.nivel_aislamiento, // Usar el nivel original pasado como argumento
        exitosas,
        fallidas,
        promedio_millis
    );

    // Imprimir resumen en logs (igual que antes)
    info!("-----------------------------------------------------");
    info!("Resumen de la Simulación:");
    info!("Usuarios Concurrentes: {}", num_usuarios);
    info!("Nivel de Aislamiento: {}", args.nivel_aislamiento);
    info!("Reservas Exitosas: {}", exitosas);
    info!("Reservas Fallidas: {}", fallidas);
    info!("Tiempo Promedio (exitosas): {} ms", promedio_millis);
    info!("-----------------------------------------------------");

    Ok(())
}
