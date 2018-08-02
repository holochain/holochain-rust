# Pairs

Pair has
  

Entry has 
  entry_type
  content


The hash of a Header is the result of combining all of its properties and then getting the hash of that string.

Header has
  entry_type
  time
  next // link to the immediately preceding header, None is valid only for genesis
  entry
  type_next // link to the most recent header of the same type, None is valid only for the first of type
  signature // agents crptographic signature




