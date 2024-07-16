---
title: A Better Merge Workflow with Jujutsu
description: A merge workflow for activating multiple branches simultaneously using Jujutsu, an alternative VCS to Git.
cover:
  image: /assets/images/cover/jujutsu-merge-workflow.jpg
publishedAt: 2024-04-10
updatedAt: 2024-07-16
---

## Introduction

Since reading [Chris Krycho's essay introduction to Jujutsu][chris-krycho-jj-init], I've been excited about the possibilities of a more modern, user-friendly Version Control System (VCS) tool. For those of you who aren't familiar, [Jujutsu][jj] (binary name: `jj`) is a new VCS which is compatible with existing Git repositories. I've been using Jujutsu as my daily Git driver for a while now. Though it's still experimental software—with its fair share of bugs and unimplemented features, it has made my day-to-day interactions with Git repositories a much more pleasant experience.

There's a really cool workflow that [Austin Seipp][aseipp] shared on Jujutsu's Discord, which I'm beginning to use everywhere, that I thought was worth writing more about. He calls it *The Austin™ Mega Merge Strategy<sup>®</sup>*, but me—I'm just going to call it for what it is: a Better Workflow for Manipulating Merge Commits in Jujutsu[^1]. This workflow makes it easy to simultaneously activate multiple branches in the same working directory.

Before I go through the workflow, let's take a look at some of the basics of Jujutsu.

## A quick primer on Jujutsu

There's no better way to demonstrate a VCS than by using it on its own repository. Jujutsu is compatible with Git repositories, and [its own repository][jj-repo] is a Git repository as well, hosted on GitHub.

Here's the terminal output of `jj log`, which displays the graph of commits in the repository:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m [36m-r[39m [36m::@[39m
@  [1m[38;5;13mytr[38;5;8muvzpy[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m11 seconds ago[39m [38;5;12me1b[38;5;8m74dba[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m1 day ago[39m [38;5;5mv0.16.0[39m [38;5;2mHEAD@git[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
◉  [1m[38;5;5mkyxs[0m[38;5;8mzyrv[39m [38;5;3m49699333+dependabot[bot]@users.noreply.github.com[39m [38;5;6m1 day ago[39m [1m[38;5;4m6826[0m[38;5;8mbe4a[39m
│  cargo: bump the cargo-dependencies group with 2 updates
◉  [1m[38;5;5mrxq[0m[38;5;8mmmmry[39m [38;5;3myuya@tcha.org[39m [38;5;6m2 days ago[39m [1m[38;5;4m363b[0m[38;5;8m5084[39m
│  cli: ditch Deref, implement AsRef and Display for RevisionArg instead
◉  [1m[38;5;5mzyn[0m[38;5;8mytznv[39m [38;5;3myuya@tcha.org[39m [38;5;6m2 days ago[39m [1m[38;5;4mc596[0m[38;5;8md457[39m
│  cli: migrate singular parse/resolve revset argument to RevisionArg
◉  [1m[38;5;5mwnkpl[0m[38;5;8mmms[39m [38;5;3myuya@tcha.org[39m [38;5;6m2 days ago[39m [1m[38;5;4m311b[0m[38;5;8mdbf5[39m
│  cli: use RevisionArg type in "resolve -r", "bench", and example command
◉  [1m[38;5;5musyx[0m[38;5;8moklz[39m [38;5;3myuya@tcha.org[39m [38;5;6m2 days ago[39m [1m[38;5;4mae91[0m[38;5;8madba[39m
│  cli: preserve RevisionArg type as much as possible
◉  [1m[38;5;5mprqkm[0m[38;5;8mmqn[39m [38;5;3myuya@tcha.org[39m [38;5;6m2 days ago[39m [1m[38;5;4m426e[0m[38;5;8me1c1[39m
│  cli: abuse Cow to declare RevisionArg("@") constant
◉  [1m[38;5;5mzrp[0m[38;5;8mqktts[39m [38;5;3mdev@noahmayr.com[39m [38;5;6m2 days ago[39m [1m[38;5;4m88a4a[0m[38;5;8m828[39m
│  cli: add better error message when immutable_heads() cannot be resolved
◉  [1m[38;5;5mnopsq[0m[38;5;8mtrw[39m [38;5;3mdev@noahmayr.com[39m [38;5;6m2 days ago[39m [1m[38;5;4mb799[0m[38;5;8m8488[39m
│  cli: only use default log revset when neither path nor revset is provided
◉  [1m[38;5;5mrrtls[0m[38;5;8muyn[39m [38;5;3milyagr@users.noreply.github.com[39m [38;5;6m2 days ago[39m [1m[38;5;4m670e[0m[38;5;8m6ac6[39m
│  cmd `squash`: alias `--to` for the `--into` flag
```

There's a couple of things to note, which differ from Git:

1. Jujutsu has separate notions of changes and commits. Changes are identified by the alphabetical IDs on the left of the log, whilst commits are identified by the hexadecimal IDs on the right. When using Jujutsu's Git backend, commits are just Git commits (the ID is the commit SHA), whereas a change is just a constant ID with an associated commit[^2].

   Change IDs address one of the pain points of Git: what we call "amending a commit" actually creates a brand new commit object with a different ID. After amending a commit, you'd need to check the status message or go back to the commit log to get the new commit ID to address the commit again.

   Using Jujutsu, "amending a commit" also produces a new commit object, as in Git, but the new commit has the same change ID as the original. The same ID always represent the same change, and points to the latest version of the commit, no matter how much you amend the commit history. This is really useful if you're doing a lot of such operations.

   (They're also really helpful in allowing you to understand how your change has evolved over time, although we won't be going through that much in this article.)

1. Jujutsu has a language called [revsets][jj-revsets] (similar to [Mercurial's revsets][mercurial-revsets]), which allows you to select commits based on given properties. `jj log -r [REVSET]` only displays the commits in the log which the revset evaluates to. `jj log -r ::@` shows all ancestors of the current working copy commit (denoted by `@`), and is the equivalent to `git log`. Revsets allow for much more succinct and expressive filters than possible in Git.

1. Jujutsu removes the concept of the index by using working copy commits. Changes made in the repository always update the working copy commit, and do not need to be separately staged and committed. Once you're done working on a change, `jj commit` will update the commit description and create a new empty change[^3], or you can use `jj commit -i` to interactively select the changes you want and split the working copy commit into two.

## Creating a new merge commit

In the repository, I've been working on a few distinct features, some of which I've already pushed to various branches in my fork of the repository. Let's take a look at the commits that I'm interested in—commits for which I'm the author, and have been pushed to a remote branch:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m [36m-r[39m [33m"mine() & remote_branches()"[39m
◉  [1m[38;5;5mnvy[0m[38;5;8mmpzuk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 day ago[39m [38;5;5mpush-nvympzukzzqo[39m [1m[38;5;4meba[0m[38;5;8mc1982[39m
│  rebase: refactor to `MutRepo::set_extracted_commit` API
~

◉  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 days ago[39m [38;5;5mpush-uqxvnturzsuu[39m [1m[38;5;4m932[0m[38;5;8me59d2[39m
│  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
~

◉  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 days ago[39m [38;5;5mpush-qklyrnvvuksv[39m [1m[38;5;4m579e[0m[38;5;8mcb73[39m
│  cli: print conflicted paths whenever the working copy is changed
~

◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m3 days ago[39m [38;5;5mssh-openssh[39m [1m[38;5;4mea9[0m[38;5;8m3486e[39m
│  git: update error message for SSH error to stop referencing libssh2
~

◉  [1m[38;5;5mxst[0m[38;5;8mwkkpp[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 month ago[39m [38;5;5mpush-syolynqvlwsl[39m [1m[38;5;4m4d3[0m[38;5;8mbb253[39m
│  git: Add revset alias for `trunk()` on `git clone`
~
```

I'd like to create a new merge commit which includes the changes from `zoz` and `qkl`. With Jujutsu, you can use `jj new` to start working on a new change, and specify any number of parent changes:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mnew[39m [36mzoz[39m [36mqkl[39m
Working copy now at: [1m[38;5;13morl[38;5;8mlnptq[39m [38;5;12m5ea[38;5;8m75c06[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [1m[38;5;4mea9[0m[38;5;8m3486e[39m [38;5;5mssh-openssh[39m[38;5;8m | [39mgit: update error message for SSH error to stop referencing libssh2
Parent commit      : [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [1m[38;5;4m579e[0m[38;5;8mcb73[39m [38;5;5mpush-qklyrnvvuksv[39m[38;5;8m | [39mcli: print conflicted paths whenever the working copy is changed
Added 0 files, modified 14 files, removed 0 files
```

This creates a new merge commit with the change ID of `orl`, with the 2 parents specified. Note that you can specify as many parents as you want, and Jujutsu can still merge them. (I'm only specifying 2 here, so I can add more later manually.) Here's what the commit graph looks like at this point:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@    [1m[38;5;13morl[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m34 seconds ago[39m [38;5;12m5ea[38;5;8m75c06[39m[0m
├─╮  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
│ ◉  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 days ago[39m [38;5;5mpush-qklyrnvvuksv[39m [1m[38;5;4m579e[0m[38;5;8mcb73[39m
│ │  cli: print conflicted paths whenever the working copy is changed
│ ~
│
◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m3 days ago[39m [38;5;5mssh-openssh[39m [38;5;2mHEAD@git[39m [1m[38;5;4mea9[0m[38;5;8m3486e[39m
│  git: update error message for SSH error to stop referencing libssh2
◉  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m3 days ago[39m [1m[38;5;4m62[0m[38;5;8m22cb24[39m
│  git: use prerelease version of `git2` with OpenSSH support
~
```

## A new merge workflow?

*Merge commits*, you might be wondering. *Is that a new workflow? Can't you just use Git for this?*

Merge commits definitely aren't anything new—nearly every modern VCS tool has merge commits. However, Jujutsu's support for manipulating the commit graph is miles ahead of Git's. With Jujutsu, you can merge commits without fear of modifying your repository to an unrecoverable state. Jujutsu's [first-class conflicts][jj-conflicts] and [`jj undo`][jj-undo] makes it safe to merge different branches, play around with different configurations of your code, and then restore your original changes.

Whether you find this article useful likely depends on how you're using your VCS right now. If you're just building a linear stack of commits, then this is probably not going to be very helpful. However, if you use separate branches to work on different features and group commits together for code review, then you might find this useful. (If you've read [Jackson Gabbard's article on Stacked Diffs vs Pull Requests][stacked-diff-vs-pr], I like to think that this workflow allows you to enjoy the benefits of a Stacked Diff-like workflow of working on a single branch, but still allows you to work with code forges like GitHub which expect a Pull Request-style workflow.)

The gist of this workflow is basically: merge all or as many of your branches/commits together as you need, and keep that combined merge commit in your working directory.

Why is this useful? Some good usecases include:
- Testing a full build of your application with different WIP features together, to see how it operates holistically as a whole
- Quickly fixing a bug or making a change without having to do the traditional Git dance of stashing changes and switching branches

Modifying existing merge commits is difficult using Git, but is much simpler with Jujutsu. Let's go through a few examples.

## Adding a new parent from an existing commit

Here's how to add another parent to the merge commit:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mrebase[39m [36m-s[39m [36morl[39m [36m-d[39m [33m"all:orl-"[39m [36m-d[39m [36mwtm[39m
Rebased 1 commits
Working copy now at: [1m[38;5;13morl[38;5;8mlnptq[39m [38;5;12mdd20[38;5;8me255[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [1m[38;5;4m579e[0m[38;5;8mcb73[39m [38;5;5mpush-qklyrnvvuksv[39m[38;5;8m | [39mcli: print conflicted paths whenever the working copy is changed
Parent commit      : [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [1m[38;5;4mea9[0m[38;5;8m3486e[39m [38;5;5mssh-openssh[39m[38;5;8m | [39mgit: update error message for SSH error to stop referencing libssh2
Parent commit      : [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [1m[38;5;4m932[0m[38;5;8me59d2[39m [38;5;5mpush-uqxvnturzsuu[39m[38;5;8m | [39mrebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
Added 0 files, modified 3 files, removed 0 files
```

This command rebases the commit with change ID `orl` and all its descendants on top of all the given destinations. The given destinations here were `all:orl-`, which means all of `orl`'s existing parents, as well as the new destination of `wtm`. We've now got a new merge commit with the 2 original parents and the new one:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@      [1m[38;5;13morl[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m20 seconds ago[39m [38;5;12mdd20[38;5;8me255[39m[0m
├─┬─╮  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
│ │ ◉  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 days ago[39m [38;5;5mpush-uqxvnturzsuu[39m [1m[38;5;4m932[0m[38;5;8me59d2[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ │ ◉  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 days ago[39m [1m[38;5;4m2ac[0m[38;5;8m431c5[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ │ ◉  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 days ago[39m [1m[38;5;4mecf[0m[38;5;8m4a6e8[39m
│ │ │  rebase: extract out some functions from `rebase_revision`
│ │ ~
│ │
│ ◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m3 days ago[39m [38;5;5mssh-openssh[39m [1m[38;5;4mea9[0m[38;5;8m3486e[39m
│ │  git: update error message for SSH error to stop referencing libssh2
│ ◉  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m3 days ago[39m [1m[38;5;4m62[0m[38;5;8m22cb24[39m
│ │  git: use prerelease version of `git2` with OpenSSH support
│ ~
│
◉  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 days ago[39m [38;5;5mpush-qklyrnvvuksv[39m [38;5;2mHEAD@git[39m [1m[38;5;4m579e[0m[38;5;8mcb73[39m
│  cli: print conflicted paths whenever the working copy is changed
~
```

If you've got any other commits on top of the one you specified for `-s`, Jujutsu also correctly rebases all of the original commit's descendants on top of the new commit, so you don't have to worry about those commits going out of sync.

## Rebasing all parents

In this case, I realized that all my feature branches were outdated, and I'd like to rebase them on top of `main`. Here's the command to do that:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mrebase[39m [36m-s[39m [33m'all:roots(main..@)'[39m [36m-d[39m [36mmain[39m
Rebased 9 commits
Working copy now at: [1m[38;5;13morl[38;5;8mlnptq[39m [38;5;12m6e4[38;5;8mf5799[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [1m[38;5;4m28a[0m[38;5;8mf9083[39m [38;5;5mpush-qklyrnvvuksv*[39m[38;5;8m | [39mcli: print conflicted paths whenever the working copy is changed
Parent commit      : [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m [38;5;5mssh-openssh*[39m[38;5;8m | [39mgit: update error message for SSH error to stop referencing libssh2
Parent commit      : [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m [38;5;5mpush-uqxvnturzsuu*[39m[38;5;8m | [39mrebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
Added 0 files, modified 3 files, removed 0 files
```

The syntax is a bit of a doozy, but can be better understood by breaking it down part-by-part:
1. `main..@`: This finds all ancestors of `@`, the working copy commit, which are not ancestors of main. (By coincidence, my default configured revset for `jj log` shows exactly all the commits in the `main..@` set.)
1. `roots(main..@)`: This gets the roots of commits in `main..@` set, which are commits that do not have any ancestors within the set. This evaluates to the first commit of each arm of the merge commit in the log above (`qkl`, `yow`, and `zoz`).
1. `all:roots(main..@)`: The `all` prefix is required since `-s` expects a single commit by default, but `roots(main..@)` evaluates to multiple commits.

Each of these 3 commits are rebased on top of the destination, `main`, and have their descendants automatically rebased as well. This results in a subgraph where the root is `main`, and the leaf is the merge commit with its 3 parents:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@      [1m[38;5;13morl[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m1 minute ago[39m [38;5;12m6e4[38;5;8mf5799[39m[0m
├─┬─╮  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
│ │ ◉  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ │ ◉  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ │ ◉  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
│ │ │  rebase: extract out some functions from `rebase_revision`
│ ◉ │  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ ◉ │  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ ├─╯  git: use prerelease version of `git2` with OpenSSH support
◉ │  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [38;5;5mpush-qklyrnvvuksv*[39m [38;5;2mHEAD@git[39m [1m[38;5;4m28a[0m[38;5;8mf9083[39m
├─╯  cli: print conflicted paths whenever the working copy is changed
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m1 day ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

Here, we've automatically rebased all the changes we're interested in with just a single command! 😲

## Adding a new parent from new changes

Whilst testing out the features from these different changes, you might want to work on a new change. Instead of having to check out a new branch as you would in Git, you can just work on the new change on top of this merge commit:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mnew[39m
Working copy now at: [1m[38;5;13mrwq[38;5;8mywnzl[39m [38;5;12m461[38;5;8md45c8[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m6e4[0m[38;5;8mf5799[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m

[0;1m[32m❯[0m [34mnvim[39m

[0;1m[32m❯[0m [34mjj[39m [36mcommit[39m [36m-m[39m [33m"new: avoid manual `unwrap()` call"[39m
Working copy now at: [1m[38;5;13movy[38;5;8mpxnus[39m [38;5;12me0c[38;5;8m160c9[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [1m[38;5;4m919[0m[38;5;8mfae76[39m new: avoid manual `unwrap()` call

[0;1m[32m❯[0m [34mjj[39m [36mshow[39m [36mrwq[39m
Commit ID: [38;5;4m919fae76dccba57d1df3df3125f4d4eac6676ce9[39m
Change ID: [38;5;5mrwqywnzlzmnoqrqkosupxwtyrxumymxs[39m
Author: Benjamin Tan <[38;5;3mbenjamin@dev.ofcr.se[39m> ([38;5;6m23 hours ago[39m)
Committer: Benjamin Tan <[38;5;3mbenjamin@dev.ofcr.se[39m> ([38;5;6m23 hours ago[39m)

    new: avoid manual `unwrap()` call

[1mdiff --git a/cli/src/commands/new.rs b/cli/src/commands/new.rs[0m
[1mindex eeeb50aee6...e0defba129 100644[0m
[1m--- a/cli/src/commands/new.rs[0m
[1m+++ b/cli/src/commands/new.rs[0m
[38;5;6m@@ -193,7 +193,7 @@[39m
             writeln!(formatter)?;
         }
     } else {
[38;5;1m-        tx.edit(&new_commit).unwrap();[39m
[38;5;2m+        tx.edit(&new_commit)?;[39m
         // The description of the new commit will be printed by tx.finish()
     }
     if num_rebased > 0 {
```

Here's the updated commit graph, with the new commit (change ID `rwq`) as a child of the merge commit:

```ansi {4,5}
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13movy[38;5;8mpxnus[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m1 minute ago[39m [38;5;12me0c[38;5;8m160c9[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4m919[0m[38;5;8mfae76[39m
│  new: avoid manual `unwrap()` call
◉      [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m15 minutes ago[39m [1m[38;5;4m6e4[0m[38;5;8mf5799[39m
├─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ ◉  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m15 minutes ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ │ ◉  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m15 minutes ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ │ ◉  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m15 minutes ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
│ │ │  rebase: extract out some functions from `rebase_revision`
│ ◉ │  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m15 minutes ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ ◉ │  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m15 minutes ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ ├─╯  git: use prerelease version of `git2` with OpenSSH support
◉ │  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m15 minutes ago[39m [38;5;5mpush-qklyrnvvuksv*[39m [1m[38;5;4m28a[0m[38;5;8mf9083[39m
├─╯  cli: print conflicted paths whenever the working copy is changed
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m1 day ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

Although this change was made on top of the merge commit, you typically wouldn't want to leave it there for long. You'd likely want to rebase it to a better location (not on top of the mega merge commit), before sending the change up for code review. For example, you can first rebase the new change onto `main`:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mrebase[39m [36m-r[39m [36mrwq[39m [36m-d[39m [36mmain[39m
Also rebased 1 descendant commits onto parent of rebased commit
Working copy now at: [1m[38;5;13movy[38;5;8mpxnus[39m [38;5;12m68ba[38;5;8mcc1f[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m6e4[0m[38;5;8mf5799[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Added 0 files, modified 1 files, removed 0 files
```

The `-r` option rebases only the given revision on top of the destination; it rebases all of its descendants on top of its parents. Effectively, this is similar to moving a commit to another location in the graph.

After rebasing onto `main`, you can then add `rwq` as a new parent of the merge commit to keep the change applied to your working directory:

```ansi {12,13}
[0;1m[32m❯[0m [34mjj[39m [36mrebase[39m [36m-s[39m [36morl[39m [36m-d[39m [33m"all:orl-"[39m [36m-d[39m [36mrwq[39m
Rebased 2 commits
Working copy now at: [1m[38;5;13movy[38;5;8mpxnus[39m [38;5;12m7f2[38;5;8m78c0d[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m7b0[0m[38;5;8m28dc9[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Added 0 files, modified 1 files, removed 0 files

[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13movy[38;5;8mpxnus[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m18 seconds ago[39m [38;5;12m7f2[38;5;8m78c0d[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉        [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m18 seconds ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4m7b0[0m[38;5;8m28dc9[39m
├─┬─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ │ ◉  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m47 seconds ago[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
│ │ │ │  new: avoid manual `unwrap()` call
│ │ ◉ │  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 minutes ago[39m [38;5;5mpush-qklyrnvvuksv*[39m [1m[38;5;4m28a[0m[38;5;8mf9083[39m
│ │ ├─╯  cli: print conflicted paths whenever the working copy is changed
│ ◉ │  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 minutes ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ ◉ │  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 minutes ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ ├─╯  git: use prerelease version of `git2` with OpenSSH support
◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 minutes ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 minutes ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │  rebase: add `--insert-after` and `--insert-before` options
◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 minutes ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
├─╯  rebase: extract out some functions from `rebase_revision`
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m1 day ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

This persists the change in the working directory, whilst extracting it to a standalone commit on top of the main branch which can be sent for code review. Here's how you can create a branch and push to an upstream repository (GitHub in this case):

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mbranch[39m [36mcreate[39m [36mtest[39m [36m-r[39m [36mrwq[39m

[32m❯[0m [34mjj[39m [36mgit[39m [36mpush[39m [36m--remote[39m [36mbnjmnt4n[39m [36m--branch[39m [36mtest[39m
Branch changes to push to bnjmnt4n:
  Add branch test to 402f7ad8b9bd
remote:
remote: Create a pull request for 'test' on GitHub by visiting:
remote:      https://github.com/bnjmnt4n/jj/pull/new/test
remote:
```

## Moving a change to a parent

Another possible scenario is that you've made some modifications to your working copy, and want to shift the commit into one of the arms of the merge commit.

This is what the commit graph looks like after making the change:

```ansi {10,11}
[0;1m[32m❯[0m [34mnvim[39m

[0;1m[32m❯[0m [34mjj[39m [36mcommit[39m [36m-m[39m [33m"test change"[39m
Working copy now at: [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;12me9f[38;5;8mebf2c[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5movy[0m[38;5;8mpxnus[39m [1m[38;5;4me99[0m[38;5;8md7578[39m misc: test change

[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m13 seconds ago[39m [38;5;12me9f[38;5;8mebf2c[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉  [1m[38;5;5movy[0m[38;5;8mpxnus[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m13 seconds ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4me99[0m[38;5;8md7578[39m
│  misc: test change
◉        [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [1m[38;5;4m7b0[0m[38;5;8m28dc9[39m
├─┬─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ │ ◉  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mtest[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
│ │ │ │  new: avoid manual `unwrap()` call
│ │ ◉ │  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mpush-qklyrnvvuksv*[39m [1m[38;5;4m28a[0m[38;5;8mf9083[39m
│ │ ├─╯  cli: print conflicted paths whenever the working copy is changed
│ ◉ │  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ ◉ │  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ ├─╯  git: use prerelease version of `git2` with OpenSSH support
◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │  rebase: add `--insert-after` and `--insert-before` options
◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
├─╯  rebase: extract out some functions from `rebase_revision`
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m1 day ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

There's a new change `ovy` which we want to set as the child of our previous change `rwq`, then update the branch `test` to point to `ovy`. There's two possible ways to do this right now using Jujutsu:
1. Rebase `ovy` onto `rwq`, rebase the merge commit to point to `ovy` instead of `rwq`, then update the branch `test` to point to `ovy`.
2. Create a new commit after `rwq`, squash the changes from `ovy` into it, then update the branch `test` to point to `ovy`.

The first way is similar to what's already been done above, so I'll show the second way of doing this. First, we insert a new commit after `rwq`, making sure to specify `--no-edit` to avoid checking out the changes in `rwq`:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mnew[39m [36m--after[39m [36mrwq[39m [36m--no-edit[39m
Created new commit [1m[38;5;5mlqksr[0m[38;5;8mtkk[39m [1m[38;5;4m6a3[0m[38;5;8m8dd7a[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Rebased 3 descendant commits
Working copy now at: [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;12m355e[38;5;8ma4ba[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5movy[0m[38;5;8mpxnus[39m [1m[38;5;4m27b[0m[38;5;8maf0ef[39m misc: test change
```

A new, empty commit with change ID `lqks` was created after `rwq`. Note how `lqks` was correctly inserted between `orl` and `rwq`, maintaining the ancestry of the merge commit:

```ansi {8,9}
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m1 minute ago[39m [38;5;12m355e[38;5;8ma4ba[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉  [1m[38;5;5movy[0m[38;5;8mpxnus[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4m27b[0m[38;5;8maf0ef[39m
│  misc: test change
◉        [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [1m[38;5;4m8c48[0m[38;5;8m6dfd[39m
├─┬─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ │ ◉  [1m[38;5;5mlqks[0m[38;5;8mrtkk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [1m[38;5;4m6a3[0m[38;5;8m8dd7a[39m
│ │ │ │  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ │ ◉  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mtest[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
│ │ │ │  new: avoid manual `unwrap()` call
│ │ ◉ │  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mpush-qklyrnvvuksv*[39m [1m[38;5;4m28a[0m[38;5;8mf9083[39m
│ │ ├─╯  cli: print conflicted paths whenever the working copy is changed
│ ◉ │  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ ◉ │  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ ├─╯  git: use prerelease version of `git2` with OpenSSH support
◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │  rebase: add `--insert-after` and `--insert-before` options
◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
├─╯  rebase: extract out some functions from `rebase_revision`
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m1 day ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

Next, we can "squash" or move the changes from `ovy` into `lqks`. This is followed by updating the branch `test` to point to `lqks`:

```ansi {13,14}
[0;1m[32m❯[0m [34mjj[39m [36msquash[39m [36m--from[39m [36movy[39m [36m--into[39m [36mlqks[39m
Rebased 2 descendant commits
Working copy now at: [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;12m23f[38;5;8m02b9f[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4mec8[0m[38;5;8m3f9fc[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m

[0;1m[32m❯[0m [34mjj[39m [36mbranch[39m [36mset[39m [36mtest[39m [36m-r[39m [36mlqks[39m

[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m1 minute ago[39m [38;5;12m23f[38;5;8m02b9f[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉        [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4mec8[0m[38;5;8m3f9fc[39m
├─┬─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ │ ◉  [1m[38;5;5mlqks[0m[38;5;8mrtkk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 minute ago[39m [38;5;5mtest*[39m [1m[38;5;4m07d[0m[38;5;8m8a576[39m
│ │ │ │  misc: test change
│ │ │ ◉  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mtest@bnjmnt4n[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
│ │ │ │  new: avoid manual `unwrap()` call
│ │ ◉ │  [1m[38;5;5mqkl[0m[38;5;8myrnvv[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [38;5;5mpush-qklyrnvvuksv*[39m [1m[38;5;4m28a[0m[38;5;8mf9083[39m
│ │ ├─╯  cli: print conflicted paths whenever the working copy is changed
│ ◉ │  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ ◉ │  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ ├─╯  git: use prerelease version of `git2` with OpenSSH support
◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │  rebase: add `--insert-after` and `--insert-before` options
◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
├─╯  rebase: extract out some functions from `rebase_revision`
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m1 day ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

The log now shows that `test@bnjmnt4n` (the branch `test` on the remote `bnjmnt4n`) points to the previous commit, whilst `test` is pointing to the commit with change ID `orl`. The `*` indicator shows that the branch has been updated, but isn't consistent with the remote.

The biggest downside of the `jj squash` workflow is that the change ID of the squashed commit is lost. You'll need to refer to the change ID of the newly created commit instead.

However, there are [plans][jj-rebase-move-commits] to improve Jujutsu to make it easier to move commits around the commit graph. In the future, a command like `jj rebase -r ovy --after rwq` might be able to move the commit whilst maintaining its change ID.

<aside>

Update: As of [Jujutsu v0.17.0][jj-v0.17.0], the `jj rebase` command has the new `--insert-after` and `--insert-before` options (short-from: `--after`/`--before`) to insert specific commits between a node and its children, or a node and its parents. The command `jj rebase -r ovy --after rwq` can now be used to move the commit `ovy` after commit `rwq` and before any of `rwq`'s children, whilst maintaining `ovy`'s' change ID.

</aside>

## Removing parents

Again, we can use `jj rebase` (and a small change to the revset) to remove parents from a merge commit:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mrebase[39m [36m-s[39m [36morl[39m [36m-d[39m [33m"all:orl- ~ qkl"[39m
Rebased 2 commits
Working copy now at: [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;12m521[38;5;8me9749[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m090[0m[38;5;8mffb0d[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Added 0 files, modified 9 files, removed 0 files
```

Previously, when adding new parents, we've specified the destinations using the flags `-d "all:orl-" -d NEW_PARENT_ID`. Now, we're specifying the destinations using `-d "all:orl- ~ qkl"`. The new argument for the destination highlights more of the revset language, in particular the set difference operator. As before, `orl-` evaluates to the set of all parents of `orl`, but `~ qkl` now subtracts `qkl` from that set.

This has the effect of removing `qkl` from the merge commit:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m14 seconds ago[39m [38;5;12m521[38;5;8me9749[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉      [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m14 seconds ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4m090[0m[38;5;8mffb0d[39m
├─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ ◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ │ ◉  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ │ │  git: use prerelease version of `git2` with OpenSSH support
│ ◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ ◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ ◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m2 hours ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
│ ├─╯  rebase: extract out some functions from `rebase_revision`
◉ │  [1m[38;5;5mlqks[0m[38;5;8mrtkk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m6 minutes ago[39m [38;5;5mtest*[39m [1m[38;5;4m07d[0m[38;5;8m8a576[39m
│ │  misc: test change
◉ │  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 hour ago[39m [38;5;5mtest@bnjmnt4n[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
├─╯  new: avoid manual `unwrap()` call
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m1 day ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

(Likewise, we could also have used set operations to add new parents to the merge commit: `jj rebase -d "all:orl- | NEW_PARENT_ID"` uses the set union operator to add the new ID to the set of existing parents.)

## Conflicting changes

What happens if you update something in the working directory, which you want to shift to a specific parent of the merge commit, but it actually conflicts with another change you made in another parent?

Here, I've committed a change which modifies the same lines as the previous commit `rwq`:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mcommit[39m [36m-m[39m [33m"conflicting change"[39m
Working copy now at: [1m[38;5;13mywr[38;5;8myozyt[39m [38;5;12m7fd[38;5;8m247f5[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5muyl[0m[38;5;8mlouwm[39m [1m[38;5;4m128[0m[38;5;8md5444[39m conflicting change

[0;1m[32m❯[0m [34mjj[39m [36mshow[39m [36muyl[39m
Commit ID: [38;5;4me8cc1f87020ecfabc4fa4b44a6a8a8d67a5de23c[39m
Change ID: [38;5;5muyllouwmkkkkrkvtzynuqwuqvxsrmpvx[39m
Author: Benjamin Tan <[38;5;3mbenjamin@dev.ofcr.se[39m> ([38;5;6m21 hours ago[39m)
Committer: Benjamin Tan <[38;5;3mbenjamin@dev.ofcr.se[39m> ([38;5;6m20 minutes ago[39m)

[38;5;3m    (no description set)[39m

[1mdiff --git a/cli/src/commands/new.rs b/cli/src/commands/new.rs[0m
[1mindex e0defba129...b946b93769 100644[0m
[1m--- a/cli/src/commands/new.rs[0m
[1m+++ b/cli/src/commands/new.rs[0m
[38;5;6m@@ -193,7 +193,8 @@[39m
             writeln!(formatter)?;
         }
     } else {
[38;5;1m-        tx.edit(&new_commit)?;[39m
[38;5;2m+        let commit = new_commit;[39m
[38;5;2m+        tx.edit(&commit).unwrap();[39m
         // The description of the new commit will be printed by tx.finish()
     }
     if num_rebased > 0 {
```

Here's the updated commit graph now, with `uyl` containing the change and no longer being empty:

```ansi {4,5}
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13mywr[38;5;8myozyt[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m35 seconds ago[39m [38;5;12m7fd[38;5;8m247f5[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉  [1m[38;5;5muyl[0m[38;5;8mlouwm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m35 seconds ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4m128[0m[38;5;8md5444[39m
│  conflicting change
◉      [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m21 hours ago[39m [1m[38;5;4m090[0m[38;5;8mffb0d[39m
├─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ ◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ │ ◉  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ │ │  git: use prerelease version of `git2` with OpenSSH support
│ ◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ ◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ ◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
│ ├─╯  rebase: extract out some functions from `rebase_revision`
◉ │  [1m[38;5;5mlqks[0m[38;5;8mrtkk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m21 hours ago[39m [38;5;5mtest*[39m [1m[38;5;4m07d[0m[38;5;8m8a576[39m
│ │  misc: test change
◉ │  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mtest@bnjmnt4n[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
├─╯  new: avoid manual `unwrap()` call
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m2 days ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

I now want to shift this new commit into the arm of the merge commit with `zoz` (the `ssh-openssh` branch), so I create a new, empty commit after `zoz`:

```ansi {14,15}
[0;1m[32m❯[0m [34mjj[39m [36mnew[39m [36m--after[39m [36mzoz[39m [36m--no-edit[39m
Created new commit [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [1m[38;5;4mae4f[0m[38;5;8mdff5[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Rebased 5 descendant commits
Working copy now at: [1m[38;5;13mywr[38;5;8myozyt[39m [38;5;12me1f[38;5;8ma0851[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5muyl[0m[38;5;8mlouwm[39m [1m[38;5;4m454[0m[38;5;8mcaf02[39m conflicting change

[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13mywr[38;5;8myozyt[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m5 seconds ago[39m [38;5;12me1f[38;5;8ma0851[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉  [1m[38;5;5muyl[0m[38;5;8mlouwm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m6 seconds ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4m454[0m[38;5;8mcaf02[39m
│  conflicting change
◉      [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m6 seconds ago[39m [1m[38;5;4mfabc[0m[38;5;8mecf1[39m
├─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ ◉  [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m6 seconds ago[39m [1m[38;5;4mae4[0m[38;5;8mfdff5[39m
│ │ │  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ ◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ │ ◉  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ │ │  git: use prerelease version of `git2` with OpenSSH support
│ ◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ ◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ ◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
│ ├─╯  rebase: extract out some functions from `rebase_revision`
◉ │  [1m[38;5;5mlqks[0m[38;5;8mrtkk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m21 hours ago[39m [38;5;5mtest*[39m [1m[38;5;4m07d[0m[38;5;8m8a576[39m
│ │  misc: test change
◉ │  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mtest@bnjmnt4n[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
├─╯  new: avoid manual `unwrap()` call
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m2 days ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

The new commit has the change ID `txs`, so I'll squash my changes from `uyl` into `txs`:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36msquash[39m [36m--from[39m [36muyl[39m [36m--into[39m [36mtxs[39m
Rebased 4 descendant commits
New conflicts appeared in these commits:
  [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [1m[38;5;4m0bb[0m[38;5;8mdad29[39m [38;5;1m(conflict)[39m conflicting change
[39mTo resolve the conflicts, start by updating to it:[39m
[39m  jj new txsrozwqlunv[39m
[39mThen use `jj resolve`, or edit the conflict markers in the file directly.[39m
[39mOnce the conflicts are resolved, you may want inspect the result with `jj diff`.[39m
[39mThen run `jj squash` to move the resolution into the conflicted commit.[39m
Working copy now at: [1m[38;5;13mywr[38;5;8myozyt[39m [38;5;12m631[38;5;8mfda4b[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m065[0m[38;5;8m31057[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
```

Jujutsu now warns that a new conflict appeared in `txs`, as expected. That's because `txs` doesn't have `rwq` in its history, which was where the first modification came from. Let's take a look at the log now:

```ansi {6,7}
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13mywr[38;5;8myozyt[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m5 seconds ago[39m [38;5;12m631[38;5;8mfda4b[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉      [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m5 seconds ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4m065[0m[38;5;8m31057[39m
├─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ ◉  [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m5 seconds ago[39m [1m[38;5;4m0bb[0m[38;5;8mdad29[39m [38;5;1mconflict[39m
│ │ │  conflicting change
│ │ ◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ │ ◉  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ │ │  git: use prerelease version of `git2` with OpenSSH support
│ ◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ ◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ ◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
│ ├─╯  rebase: extract out some functions from `rebase_revision`
◉ │  [1m[38;5;5mlqks[0m[38;5;8mrtkk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m21 hours ago[39m [38;5;5mtest*[39m [1m[38;5;4m07d[0m[38;5;8m8a576[39m
│ │  misc: test change
◉ │  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mtest@bnjmnt4n[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
├─╯  new: avoid manual `unwrap()` call
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m2 days ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

The commit `txs` is marked in the log as containing a conflict. Here's what `txs` looks like:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mshow[39m [36mtxs[39m
Commit ID: [38;5;4m0bbdad290b695b94aab4e973349e1a5dda6ef0ce[39m
Change ID: [38;5;5mtxsrozwqlunvppzymmtrnvotvtrnuwxr[39m
Author: Benjamin Tan <[38;5;3mbenjamin@dev.ofcr.se[39m> ([38;5;6m14 minutes ago[39m)
Committer: Benjamin Tan <[38;5;3mbenjamin@dev.ofcr.se[39m> ([38;5;6m13 minutes ago[39m)

    conflicting change

[1mdiff --git a/cli/src/commands/new.rs b/cli/src/commands/new.rs[0m
[1mindex eeeb50aee6...0000000000 100644[0m
[1m--- a/cli/src/commands/new.rs[0m
[1m+++ b/cli/src/commands/new.rs[0m
[38;5;6m@@ -193,7 +193,14 @@[39m
             writeln!(formatter)?;
         }
     } else {
[38;5;1m-        tx.edit(&new_commit).unwrap();[39m
[38;5;2m+<<<<<<<[39m
[38;5;2m+%%%%%%%[39m
[38;5;2m+-        tx.edit(&new_commit)?;[39m
[38;5;2m++        tx.edit(&new_commit).unwrap();[39m
[38;5;2m++++++++[39m
[38;5;2m+        let commit = new_commit;[39m
[38;5;2m+        tx.edit(&commit).unwrap();[39m
[38;5;2m+>>>>>>>[39m
         // The description of the new commit will be printed by tx.finish()
     }
     if num_rebased > 0 {
```

The original line from `txs`'s parent commit in red is replaced with the new conflicting changes in green. Jujutsu's [conflict markers][jj-conflict-markers] are slightly different from Git: lines following `%%%%%%%` are a diff between 2 sides, whilst lines following `+++++++` are a snapshot of the changes a side. Here's my annotations on what the conflict markers mean:

```plaintext {3}#del {4}#ins {6,7}
<<<<<<<
%%%%%%% Diff from destination `txs` to base tree of `orl`
-        tx.edit(&new_commit)?;
+        tx.edit(&new_commit).unwrap();
+++++++ Snapshot of new changes from source `uyl`
        let commit = new_commit;
        tx.edit(&commit).unwrap();
>>>>>>>
```

Even though `txs` has a conflict, note that the merge commit `orl` isn't in a conflicted state. This is because Jujutsu doesn't just store conflict markers, but the full metadata of the conflicts, so it can resolve the conflicts by applying all the changes from each of `orl`'s parents.

However, if we want to update the `ssh-openssh` branch to include the changes in `txs`, we can't just push a conflicted file since it won't be accepted in any code review. We need to first resolve the conflict in `txs`. I'm doing this manually here by checking out `txs` and editing the file in the working directory, but you can also use a graphical tool for conflict resolution.

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mnew[39m [36mtxs[39m
Working copy now at: [1m[38;5;13mysw[38;5;8mompum[39m [38;5;12m3ea[38;5;8m0e8e9[39m [38;5;9m(conflict)[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [1m[38;5;4m0bb[0m[38;5;8mdad29[39m [38;5;1m(conflict)[39m conflicting change
Added 0 files, modified 5 files, removed 0 files

[0;1m[32m❯[0m [34mnvim[39m

[0;1m[32m❯[0m [34mjj[39m [36mstatus[39m
Working copy changes:
[38;5;6mM cli/src/commands/new.rs[39m
Working copy : [1m[38;5;13mysw[38;5;8mompum[39m [38;5;12m509[38;5;8mf7ea1[39m [38;5;3m(no description set)[0m
Parent commit: [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [1m[38;5;4m0bb[0m[38;5;8mdad29[39m [38;5;1m(conflict)[39m conflicting change
```

The changes are updated in my working copy commit, so I can squash the changes into `txs` to apply the resolution there as well:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36msquash[39m
Rebased 3 descendant commits
Existing conflicts were resolved or abandoned from these commits:
  [1m[39mtxs[0m[38;5;8mrozwq[39m hidden [1m[38;5;4m0bbd[0m[38;5;8mad29[39m [38;5;1m(conflict)[39m conflicting change
Working copy now at: [1m[38;5;13mxmy[38;5;8mnmysw[39m [38;5;12m53a[38;5;8m37139[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [1m[38;5;4ma11[0m[38;5;8m303a3[39m conflicting change
```

So, `txs` no longer has any conflicts, and we can update our branch to point to it and push it for review. However, if we go back to our merge commit `orl`, we can see that the merge commit is now marked as conflicting:

```ansi {9,10}
[0;1m[32m❯[0m [34mjj[39m [36mnew[39m [36morl[39m
Working copy now at: [1m[38;5;13mnlw[38;5;8mnwups[39m [38;5;12m69a[38;5;8m1f4ce[39m [38;5;9m(conflict)[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m919[0m[38;5;8m2cf35[39m [38;5;1m(conflict)[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Added 0 files, modified 5 files, removed 0 files

[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13mnlw[38;5;8mnwups[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m1 minute ago[39m [38;5;12m69a[38;5;8m1f4ce[39m [38;5;9mconflict[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉      [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m5 minutes ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4m919[0m[38;5;8m2cf35[39m [38;5;1mconflict[39m
├─┬─╮  [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
│ │ ◉  [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m5 minutes ago[39m [1m[38;5;4ma11[0m[38;5;8m303a3[39m
│ │ │  conflicting change
│ │ ◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 day ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ │ ◉  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 day ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ │ │  git: use prerelease version of `git2` with OpenSSH support
│ ◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 day ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ ◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 day ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ ◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 day ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
│ ├─╯  rebase: extract out some functions from `rebase_revision`
◉ │  [1m[38;5;5mlqks[0m[38;5;8mrtkk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m23 hours ago[39m [38;5;5mtest*[39m [1m[38;5;4m07d[0m[38;5;8m8a576[39m
│ │  misc: test change
◉ │  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m1 day ago[39m [38;5;5mtest@bnjmnt4n[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
├─╯  new: avoid manual `unwrap()` call
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m2 days ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~
```

This makes sense, because we've removed manually resolved the conflicts from `txs`. Jujutsu no longer has the metadata about how the conflict came about from merging different files, so `orl` now has a conflict. Typically, this isn't that big an issue since you can just delay conflict resolution for that individual commit until you're done working on that branch. You can then remove that branch from the merge commit after that.

If you do want to continue working on both branches, you can also restore the state of the working directory to its original state, before squashing the commit. First, get the commit ID of the change `uyl` that we wrote originally, before any commit manipulation (`128d5444` from above), then run `jj restore`:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mrestore[39m [36m--from[39m [36m128d5444[39m [36m--to[39m [36morl[39m
Created [1m[38;5;5morlln[0m[38;5;8mptq[39m [1m[38;5;4mf2388[0m[38;5;8m131[39m [38;5;3m(no description set)[39m
Rebased 1 descendant commits
Working copy now at: [1m[38;5;13mnlw[38;5;8mnwups[39m [38;5;12md37[38;5;8mfb681[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4mf23[0m[38;5;8m88131[39m [38;5;3m(no description set)[39m
Added 0 files, modified 1 files, removed 0 files
```

This restores all the files in commit `orl` to their state in commit `128d5444`—the original files before the conflict occured due to squashing of commits. This has the effect of solving the conflict within the merge commit `orl`:

```ansi {4,5}
[0;1m[32m❯[0m [34mjj[39m [36mlog[39m
@  [1m[38;5;13mnlw[38;5;8mnwups[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;14m39 seconds ago[39m [38;5;12md37[38;5;8mfb681[39m[0m
│  [1m[38;5;10m(empty)[39m [38;5;10m(no description set)[39m[0m
◉      [1m[38;5;5morl[0m[38;5;8mlnptq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m39 seconds ago[39m [38;5;2mHEAD@git[39m [1m[38;5;4mf23[0m[38;5;8m88131[39m
├─┬─╮  [38;5;3m(no description set)[39m
│ │ ◉  [1m[38;5;5mtxs[0m[38;5;8mrozwq[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m3 days ago[39m [1m[38;5;4ma11[0m[38;5;8m303a3[39m
│ │ │  conflicting change
│ │ ◉  [1m[38;5;5mzoz[0m[38;5;8mvwmow[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m4 days ago[39m [38;5;5mssh-openssh*[39m [1m[38;5;4mc6c[0m[38;5;8m73906[39m
│ │ │  git: update error message for SSH error to stop referencing libssh2
│ │ ◉  [1m[38;5;5myow[0m[38;5;8mkkkqn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m4 days ago[39m [1m[38;5;4mffe[0m[38;5;8mc92c9[39m
│ │ │  git: use prerelease version of `git2` with OpenSSH support
│ ◉ │  [1m[38;5;5mwtm[0m[38;5;8mqulxn[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m4 days ago[39m [38;5;5mpush-uqxvnturzsuu*[39m [1m[38;5;4m867[0m[38;5;8m3733e[39m
│ │ │  rebase: allow both `--insert-after` and `--insert-before` to be used simultaneously
│ ◉ │  [1m[38;5;5muqx[0m[38;5;8mvntur[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m4 days ago[39m [1m[38;5;4mdd7[0m[38;5;8m454a2[39m
│ │ │  rebase: add `--insert-after` and `--insert-before` options
│ ◉ │  [1m[38;5;5mnkzsq[0m[38;5;8mppm[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m4 days ago[39m [1m[38;5;4m0a94[0m[38;5;8m9714[39m
│ ├─╯  rebase: extract out some functions from `rebase_revision`
◉ │  [1m[38;5;5mlqks[0m[38;5;8mrtkk[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m4 days ago[39m [38;5;5mtest*[39m [1m[38;5;4m07d[0m[38;5;8m8a576[39m
│ │  misc: test change
◉ │  [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [38;5;3mbenjamin@dev.ofcr.se[39m [38;5;6m4 days ago[39m [38;5;5mtest@bnjmnt4n[39m [1m[38;5;4m402[0m[38;5;8mf7ad8[39m
├─╯  new: avoid manual `unwrap()` call
◉  [1m[38;5;5moqtz[0m[38;5;8mskyx[39m [38;5;3mmartinvonz@google.com[39m [38;5;6m5 days ago[39m [38;5;5mmain*[39m [38;5;5mv0.16.0[39m [1m[38;5;4m2dcd[0m[38;5;8mc7fb[39m
│  release: release version 0.16.0
~

[0;1m[32m❯[0m [34mjj[39m [36mshow[39m [36morl[39m
Commit ID: [38;5;4mf2388131cba4be8fe0b267dcef1af8d823184851[39m
Change ID: [38;5;5morllnptqzkuqpsonzzkytxlrzpyxmwtn[39m
Author: Benjamin Tan <[38;5;3mbenjamin@dev.ofcr.se[39m> ([38;5;6m4 days ago[39m)
Committer: Benjamin Tan <[38;5;3mbenjamin@dev.ofcr.se[39m> ([38;5;6m1 minute ago[39m)

[38;5;3m    (no description set)[39m

[1mdiff --git a/cli/src/commands/new.rs b/cli/src/commands/new.rs[0m
[1mindex 0000000000...b946b93769 100644[0m
[1m--- a/cli/src/commands/new.rs[0m
[1m+++ b/cli/src/commands/new.rs[0m
[38;5;6m@@ -193,14 +193,8 @@[39m
             writeln!(formatter)?;
         }
     } else {
[38;5;1m-<<<<<<<[39m
[38;5;1m-%%%%%%%[39m
[38;5;1m--        tx.edit(&new_commit).unwrap();[39m
[38;5;1m-+        tx.edit(&new_commit)?;[39m
[38;5;1m-+++++++[39m
         let commit = new_commit;
         tx.edit(&commit).unwrap();
[38;5;1m->>>>>>>[39m
         // The description of the new commit will be printed by tx.finish()
     }
     if num_rebased > 0 {
```

## Conclusion

I've shown how you can use Jujutsu to manipulate merge commits and work on separate logical branches of code, by adding and removing parents using the `jj rebase` command. Arguably, this workflow might be better visualized with a graphical interface to see how the commit graph is being manipulated. There's a GUI app for Jujutsu, called [GG][gg], with a [pretty convincing demo][gg-demo] of this workflow.

Working on multiple branches of code at the same time can be really powerful. Personally, I use this workflow all the time to avoid having to switch branches. This is especially convenient when working on small bugfixes where it's definitely easier to just work in the current directory.

Merging multiple branches together also allows you to very simply test out various features at the same time, without having to wait for all of them to be merged into the main branch. In fact, I use this to build a custom `jj` binary which contains various features which haven't been merged into `main`.

Even if you aren't convinced about switching to Jujutsu, I think this workflow is still valuable. In fact, [GitButler][gitbutler]—a new Git client—was recently launched with a similar end product: making it easy to activate different "virtual branches" in your working directory. Otherwise, alternative tools like [git-branchless][git-branchless] might allow you to do something similar.

If you are intrigued by Jujutsu, do check out the [introduction][jj-intro] and [tutorial][jj-tutorial]. I'd also recommend [Chris's article][chris-krycho-jj-init] and [video series][chris-krycho-video-series]. Steve Klabnik also has a [long-form tutorial][steve-klabnik-tutorial] on Jujutsu, which includes a [chapter on this workflow][steve-klabnik-merge-workflow].


[^1]: Hmmm, maybe *The Austin™ Mega Merge Strategy<sup>®</sup>* is the better name after all...
[^2]: Actually, you *can* have change IDs associated with multiple commits, but that's out of the scope of this article.
[^3]: `jj commit` is a shorthand for `jj describe`, to describe the current change, and `jj new`, to create a new change.

[chris-krycho-jj-init]: https://v5.chriskrycho.com/essays/jj-init/
[jj]: https://martinvonz.github.io/jj/
[aseipp]: https://www.austinseipp.com/
[jj-repo]: https://github.com/martinvonz/jj
[jj-revsets]: https://martinvonz.github.io/jj/latest/revsets/
[mercurial-revsets]: https://repo.mercurial-scm.org/hg/help/revsets
[jj-conflicts]: https://martinvonz.github.io/jj/latest/conflicts/
[jj-undo]: https://martinvonz.github.io/jj/latest/operation-log/
[stacked-diff-vs-pr]: https://jg.gg/2018/09/29/stacked-diffs-versus-pull-requests/
[jj-rebase-move-commits]: https://github.com/martinvonz/jj/issues/1188
[jj-v0.17.0]: https://github.com/martinvonz/jj/releases/tag/v0.17.0
[jj-conflict-markers]: https://martinvonz.github.io/jj/latest/conflicts/#conflict-markers
[gitbutler]: https://gitbutler.com/
[git-branchless]: https://github.com/arxanas/git-branchless
[jj-intro]: https://github.com/martinvonz/jj#introduction
[jj-tutorial]: https://martinvonz.github.io/jj/latest/tutorial/
[chris-krycho-video-series]: https://www.youtube.com/playlist?list=PLelyiwKWHHAq01Pvmpf6x7J0y-yQpmtxp
[steve-klabnik-tutorial]: https://steveklabnik.github.io/jujutsu-tutorial/
[steve-klabnik-merge-workflow]: https://steveklabnik.github.io/jujutsu-tutorial/advanced/simultaneous-edits.html
[gg]: https://github.com/gulbanana/gg
[gg-demo]: https://www.youtube.com/watch?v=cD9L3Mi1Vy4
