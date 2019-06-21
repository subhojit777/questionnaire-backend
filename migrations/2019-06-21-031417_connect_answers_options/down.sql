ALTER TABLE answers ADD title VARCHAR(255) NOT NULL;
ALTER TABLE answers ADD question_id INT NOT NULL;
ALTER TABLE answers DROP option_id;
