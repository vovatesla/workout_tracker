CREATE TABLE workouts (
    id BIGSERIAL PRIMARY KEY,
    date TEXT NOT NULL,
    muscle_group TEXT NOT NULL,
    notes TEXT
);