CREATE TABLE answers (
  id INT PRIMARY KEY,
  question_id INT NOT NULL,
  title VARCHAR(255) NOT NULL,
  user_id INT NOT NULL,
  created INT NOT NULL
)
