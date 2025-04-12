-- Archivo: proyecto2/01_ddl.sql (CORREGIDO)

-- Crear la tabla de eventos
CREATE TABLE eventos (
    id SERIAL PRIMARY KEY,
    nombre TEXT NOT NULL
);

-- Crear la tabla de asientos
CREATE TABLE asientos (
    id SERIAL PRIMARY KEY,
    evento_id INTEGER NOT NULL REFERENCES eventos(id),
    numero_asiento TEXT NOT NULL
);

-- Crear la tabla de reservas CORREGIDA
-- Elimina cualquier definición anterior de 'reservas' si es necesario (DROP TABLE IF EXISTS reservas;)
CREATE TABLE reservas (
    id SERIAL PRIMARY KEY,
    asiento_id INTEGER NOT NULL UNIQUE REFERENCES asientos(id),
    -- nombre_cliente TEXT NOT NULL, -- Columna eliminada o comentada
    usuario_id INTEGER NOT NULL,
    timestamp_reserva TIMESTAMP WITH TIME ZONE NOT NULL -- Columna añadida
);
