REMOVE TABLE books;
REMOVE TABLE authors;
REMOVE TABLE wrote;

LET $harper_lee = (CREATE authors CONTENT { name: "Harper Lee" } RETURN id);
RELATE $harper_lee->wrote->(INSERT INTO books (`name`, `word_count`) VALUES
  ("To Kill a Mockingbird", 100388)
);

LET $charles_dickens = (CREATE authors CONTENT { name: "Charles Dickens"} RETURN id);
RELATE $charles_dickens->wrote->(INSERT INTO books (`name`, `word_count`) VALUES
  ("Bleak House", 360947),
  ("Great Expectations", 183349)
);

LET $leo_tolstoy = (CREATE authors CONTENT { name: "Leo Tolstoy" } RETURN id);
RELATE $leo_tolstoy->wrote->(INSERT INTO books (`name`, `word_count`) VALUES
  ("War and Peace", 561304),
  ("Anna Karenina", 349736)
);

LET $grrm = (CREATE authors CONTENT { name: "George R. R. Martin" } RETURN id);
RELATE $grrm->wrote->(INSERT INTO books (`name`, `word_count`) VALUES
  ("A Game of Thrones", 298000),
  ("A Clash of Kings", 326000),
  ("A Storm of Swords", 424000),
  ("A Feast for Crows", 300000)
);

LET $jrrt = (CREATE authors CONTENT { name: "J. R. R. Tolkien"} RETURN id);
RELATE $jrrt->wrote->(INSERT INTO books (`name`, `word_count`) VALUES
  ("The Hobbit", 95356),
  ("The Fellowship of the Ring", 187790),
  ("The Two Towers", 156198),
  ("The Return of the King", 137115)
);

LET $philip = (CREATE authors CONTENT { name: "Philip Pullman" } RETURN id);
RELATE $philip->wrote->(INSERT INTO `books` (`name`, `word_count`) VALUES
  ("The Golden Compass", 112_815),
  ("The Subtle Knife", 109_120),
  ("The Amber Spyglass", 168640)
);

LET $susanna = (CREATE authors CONTENT { name: "Susanna Clarke" } RETURN id);
RELATE $susanna->wrote->(INSERT INTO `books` (`name`, `word_count`) VALUES
  ("Jonathan Strange & Mr Norrell", 308931)
);