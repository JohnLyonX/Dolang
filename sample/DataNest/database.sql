-- 创建数据库
CREATE DATABASE datanest;

-- 连接数据库
\c datanest;

-- 用户表
CREATE TABLE users (
                       id          SERIAL PRIMARY KEY,
                       username    VARCHAR(50)  NOT NULL UNIQUE,
                       email       VARCHAR(100) NOT NULL UNIQUE,
                       password    VARCHAR(255) NOT NULL,          -- 存 bcrypt hash
                       created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                       updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 标签表
CREATE TABLE tags (
                      id          SERIAL PRIMARY KEY,
                      name        VARCHAR(50) NOT NULL,
                      user_id     INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                      created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                      UNIQUE(name, user_id)                        -- 同一用户标签不重名
);

-- 笔记表
CREATE TABLE notes (
                       id          SERIAL PRIMARY KEY,
                       title       VARCHAR(200) NOT NULL,
                       content     TEXT,
                       user_id     INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                       created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                       updated_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- 笔记与标签的关联表（多对多）
CREATE TABLE note_tags (
                           note_id     INT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
                           tag_id      INT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
                           PRIMARY KEY (note_id, tag_id)
);