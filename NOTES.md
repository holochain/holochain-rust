AddLink is always validated now, but should it be?
- If AddLink entry is always valid, we could refer to its address, but only if we knew what "type" it was (add/remove)
- If it is not valid if the base does not exist, then we defer holding it, make it pending, and just run the whole workflow again when the base comes in. This seems like the way to go. 

Realizing I should wait until RemoveLink is done!
