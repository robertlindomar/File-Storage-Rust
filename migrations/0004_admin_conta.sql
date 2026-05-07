-- Conta de administrador unica (login com email + palavra-passe; JWT nas rotas /admin/*).

CREATE TABLE admin_conta (
    id SMALLINT PRIMARY KEY DEFAULT 1,
    email TEXT NOT NULL UNIQUE,
    senha_hash TEXT NOT NULL,
    atualizado_em TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT admin_conta_singleton CHECK (id = 1)
);
