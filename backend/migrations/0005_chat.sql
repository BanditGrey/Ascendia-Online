-- Chat é persistido para moderação/auditoria; Redis mantém somente o histórico quente.
CREATE TABLE chat_messages (
    id uuid PRIMARY KEY,
    sender_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_user_id uuid REFERENCES users(id) ON DELETE CASCADE,
    channel varchar(16) NOT NULL CHECK (channel IN ('global','whisper')),
    content varchar(280) NOT NULL CHECK (char_length(btrim(content)) BETWEEN 1 AND 280),
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK ((channel = 'global' AND recipient_user_id IS NULL) OR (channel = 'whisper' AND recipient_user_id IS NOT NULL))
);
CREATE INDEX chat_global_history ON chat_messages(created_at DESC) WHERE channel = 'global';
CREATE INDEX chat_recipient_history ON chat_messages(recipient_user_id, created_at DESC) WHERE channel = 'whisper';

CREATE TABLE user_blocks (
    user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    blocked_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, blocked_user_id),
    CHECK (user_id <> blocked_user_id)
);

CREATE TABLE chat_reports (
    id uuid PRIMARY KEY,
    reporter_user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    message_id uuid NOT NULL REFERENCES chat_messages(id) ON DELETE CASCADE,
    reason varchar(280) NOT NULL CHECK (char_length(btrim(reason)) BETWEEN 3 AND 280),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (reporter_user_id, message_id)
);
