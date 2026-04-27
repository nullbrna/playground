CREATE TABLE IF NOT EXISTS users (
    id         SERIAL       PRIMARY KEY,
    first_name VARCHAR(100) NOT NULL,
    last_name  VARCHAR(100) NOT NULL,
    age        INTEGER      NOT NULL,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users (first_name, last_name, age) VALUES
    ('John',        'Smith',     34),
    ('Jane',        'Johnson',   28),
    ('Michael',     'Williams',  42),
    ('Emily',       'Brown',     31),
    ('David',       'Jones',     55),
    ('Sarah',       'Garcia',    26),
    ('Robert',      'Miller',    39),
    ('Jessica',     'Davis',     33),
    ('William',     'Rodriguez', 47),
    ('Ashley',      'Martinez',  29),
    ('James',       'Anderson',  51),
    ('Jennifer',    'Taylor',    24),
    ('Christopher', 'Thomas',    36),
    ('Amanda',      'Hernandez', 43),
    ('Matthew',     'Moore',     30),
    ('Stephanie',   'Martin',    38),
    ('Daniel',      'Jackson',   27),
    ('Michelle',    'Thompson',  45),
    ('Joshua',      'White',     32),
    ('Lauren',      'Lopez',     41);
