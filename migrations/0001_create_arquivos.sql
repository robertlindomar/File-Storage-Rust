CREATE TABLE IF NOT EXISTS arquivos (
    id UUID PRIMARY KEY,
    nome_arquivo TEXT NOT NULL,
    caminho TEXT NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_arquivos_criado_em ON arquivos (criado_em DESC);
