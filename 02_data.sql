-- Archivo: proyecto2/02_data.sql (CORREGIDO)

-- Insertar datos iniciales en eventos
INSERT INTO eventos (nombre) VALUES
('Concierto Rock'),
('Obra de Teatro');

-- Insertar datos iniciales en asientos (asociados al primer evento)
INSERT INTO asientos (evento_id, numero_asiento) VALUES
(1, 'A1'), (1, 'A2'), (1, 'A3'), (1, 'A4'), (1, 'A5'),
(1, 'B1'), (1, 'B2'), (1, 'B3'), (1, 'B4'), (1, 'B5');

-- Insertar datos iniciales en reservas (CORREGIDO)
-- Se eliminó nombre_cliente y se añadió timestamp_reserva
INSERT INTO reservas (asiento_id, usuario_id, timestamp_reserva)
VALUES
    (1, 1001, NOW()), -- Usamos NOW() para obtener el timestamp actual
    (2, 1002, NOW()),
    (3, 1003, NOW()),
    (4, 1004, NOW()),
    (5, 1005, NOW()),
    (6, 1006, NOW()),
    (7, 1007, NOW()),
    (8, 1008, NOW()),
    (9, 1009, NOW()),
    (10, 1010, NOW());
