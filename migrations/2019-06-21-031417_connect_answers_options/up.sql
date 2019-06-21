ALTER TABLE answers DROP title;
ALTER TABLE answers DROP question_id;
ALTER TABLE answers ADD option_id INT NOT NULL;
