# What's the idea?

Frame of Reference is compression method based on relativity between numbers. Assuming that we use as input an stream of data which contains **only** numbers, we can find the minimum value among them and, compute the difference between the rest against the min. So, let's say that we have:

```
20, 13, 100, 42
```

We compute min as `13` and find the difference between `20, 100, 42`. So, we'll end up with `7, 87, 29`. Notice two things here, the firs one is that numbers are always postives, and second one is that we need less bites to store the "result".

