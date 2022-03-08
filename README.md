# What's the idea?

Frame of Reference is compression method based on relativity between numbers. Assuming that we use as input a stream of data which contains **only** numbers (*RawNumbers*), we can find the minimum value among them and compute the difference between the rest against the min. So, let's say that we have:

```
20, 13, 100, 42
```

We compute min as `13` (*reference*) and find the difference between `20, 100, 42`. So, we'll end up with `7, 87, 29`. Notice two things here, the first one is that numbers are always positives, and second one is that we need less bits to store the "substracted numbers" since they're smaller (closer to zero).
