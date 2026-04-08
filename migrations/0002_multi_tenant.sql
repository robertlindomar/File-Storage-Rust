-- Evolucao multi-tenant: projetos com API keys e arquivos isolados por projeto.
-- Dados antigos da tabela `arquivos` (sem projeto) sao descartados.

DROP TABLE IF EXISTS arquivos;

CREATE TABLE projetos (
    id UUID PRIMARY KEY,
    nome TEXT NOT NULL,
    api_key TEXT NOT NULL UNIQUE,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_projetos_criado_em ON projetos (criado_em DESC);

CREATE TABLE arquivos (
    id UUID PRIMARY KEY,
    projeto_id UUID NOT NULL REFERENCES projetos (id) ON DELETE CASCADE,
    nome_arquivo TEXT NOT NULL,
    caminho TEXT NOT NULL,
    tipo_mime TEXT NOT NULL,
    tamanho BIGINT NOT NULL,
    criado_em TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_arquivos_projeto_criado ON arquivos (projeto_id, criado_em DESC);
CREATE INDEX idx_arquivos_projeto_id ON arquivos (projeto_id, id);
