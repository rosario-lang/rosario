*WARNING: This README is not complete.*

# Rosario Programming Language

## Empowering Safety with Trees!

Rosario aims to be a strongly typed, blazingly fast and fully safe programming language, helping you to focus into what's important: build and finish fast, safe, and great software.

### You should focus on your goals, not on the language's intricacies.

We all like features (and lack of features) based around memory management. Some will like garbage collectors, others will like manual memory management, others will like the borrow checker, and so on... But those can become a problem when you're trying to finish a product, and the "so beloved" features start getting in the way:

- We accidentally used after free one or more times, and we don't know where.
- The borrow checker shows up like a jumpscare and all of the sudden we have to deal with its rules.
- The garbage collector isn't working correctly and its causing memory leaks that are hard to workaround.
- We caused undefined behavior somewhere in a project with a minimum of +10.000 lines of code.

And the list goes on...

These are issues that can (and will) happen in the middle of your project, and it will force you to shift your focus COMPLETELY into having to deal with a language related problem, that it'll not allow you to continue the project, until you fix it.

Rosario aims to help you with that, so you can focus on what's important: **Accomplishing your goals**... That doesn't mean that you can't focus on the language's features, **but it should've be a choice and NOT something that you're forced to**.

### Mistakes and Misunderstandings shouldn't have exaggerated consequences.

Sure, no language is perfect (this one included), and no human is perfect either... **However**, the consequence of making a simple mistake shouldn't be extremely high, like being forced to rewrite months of work; or in the worst case scenario, being forced to rewrite the entire project altogether, just because you've made a mistake in your architechture that, in theory, should've be something simple to fix.

Something that fits in this category can be known as an "anti-pattern". An anti-pattern is coding something in a way that experienced programmers of *"insert language here"* would tell you to NOT CODE IT LIKE THAT AT ALL, but you can, because it follows all of the rules set by the language, so compiler and runtime errors don't show up. If you're not experienced enough, you can accidentally start writing them, resulting in you slowly, CORRECTLY and in some languages, SAFELY screwing it up.

These anti-patterns will not show up as a problem instantly: the more you code them, the more they will evolve slowly in the shadows, and how big or small of a problem these are going to be is based on pure luck. And before you know it, **BAM!** all of the sudden they start causing problems in the worst possible time, at the worst possible place, and its too big and too late to deal with them.

The worst thing about these, is that when they finally show up as a problem, they first disguise themselves as a simple issue... But the moment you try to fix them, they blow up like if you stepped on a land mine.

This is something that happens way frequently than it should, and the solution that experienced programmers of *"insert language here"* will tell you is: "Well, if you didn't code it like that, you wouldn't have to rewrite it."... But nobody wants to hear that at 2AM, with 4 hours left to the deadline, with their entire blood stream being replaced with coffee.

One of Rosario's goals is to fix this issue by stopping you as soon as possible, in form of compiler errors, before you accidentally write an anti-pattern. And if you manage to write one, the language will help you get rid of them as FAST as possible, so you can quickly get back on track and finish your task.

*==============*
*TODO: Continue*
*==============*
