---
title: A Better Merge Workflow with Jujutsu
description: A merge workflow for activating multiple branches simultaneously using Jujutsu, an alternative VCS to Git.
cover:
  image: /src/assets/images/jujutsu-merge-workflow.jpg
---

## Introduction

Since reading [Chris Krycho's essay introduction to Jujutsu][chris-krycho-jj-init], I've been excited about the possibilities of a more modern, user-friendly Version Control System (VCS) tool. For those of you who aren't familiar, [Jujutsu][jj] (binary name: `jj`) is a new VCS which is compatible with existing Git repositories. I've been using Jujutsu as my daily Git driver for a while now. Though it's still experimental software—with its fair share of bugs and unimplemented features, it has made my day-to-day interactions with Git repositories a much more pleasant experience.

There's a really cool workflow that [Austin Seipp][aseipp] shared on Jujutsu's Discord, which I'm beginning to use everywhere, that I thought was worth writing more about. He calls it *The Austin™ Mega Merge Strategy<sup>®</sup>*, but me—I'm just going to call it for what it is: a Better Workflow for Manipulating Merge Commits in Jujutsu[^1].

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

   Using Jujutsu, "amending a commit" also produces a new commit object, as in Git, but the new commit has the same change ID as the original. This means you always have a constant ID to represent the same change, no matter how much you amend the commit history. This is really useful if you're doing a lot of such operations.

1. Jujutsu has a language called [revsets][jj-revsets] (similar to [Mercurial][mercurial]'s revsets), which allows you to select commits based on given properties. `jj log -r [REVSET]` only displays the commits in the log which the revset evaluates to. `jj log -r ::@` shows all ancestors of the current working copy commit (denoted by `@`), and is the equivalent to `git log`. Revsets allow for much more succinct and expressive filters than possible in Git.

## Creating a new merge commit

I've been working on a few distinct features, some of which I've already pushed to various branches in my fork of the repository. Let's take a look at the commits that I'm interested in—commits for which I'm the author, and which have pushed to a remote branch:

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

We've got a new merge commit with the change ID of `orl`, with the 2 parents specified. Note that you can specify as many parents as you want, and Jujutsu can still merge them. (I'm only specifying 2 here, so I can add more later.) Here's what the commit graph looks like at this point:

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

Merge commits definitely aren't anything new—nearly every modern VCS tool has merge commits. However, Jujutsu's tooling and support for manipulating the commit graph is miles ahead of Git's. With Jujutsu, you can merge commits without fear of modifying your repository to an unrecoverable state. Jujutsu's [first-class conflicts][jj-conflicts] and [`jj undo`][jj-undo] makes it safe to merge different branches, play around with different configurations of your code, and then restore your original changes.

Whether you find this article useful likely depends on how you're using your VCS right now. If you're just building a linear stack of commits, then this is probably not going to be very helpful. However, if you use separate branches to work on different features and group commits together for code review, then you might find this useful.

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

Whilst testing out the features from these different changes, you might want to work on a new change. Instead of having to check out a new branch as you would in Git, you can just work on the new change on top of this merge commit.

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mnew[39m
Working copy now at: [1m[38;5;13mrwq[38;5;8mywnzl[39m [38;5;12m461[38;5;8md45c8[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m6e4[0m[38;5;8mf5799[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m

[0;1m[32m❯[0m [34mnvim[39m

[0;1m[32m❯[0m [34mjj[39m [36mcommit[39m [36m-m[39m [33m"new: avoid manual `unwrap()` call"[39m
Working copy now at: [1m[38;5;13movy[38;5;8mpxnus[39m [38;5;12me0c[38;5;8m160c9[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5mrwq[0m[38;5;8mywnzl[39m [1m[38;5;4m919[0m[38;5;8mfae76[39m new: avoid manual `unwrap()` call
```

Here's the updated commit graph, with the new commit (change ID `rwq`) as a child of the merge commit:

```ansi
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

Although this change was made on top of the merge commit, you typically wouldn't want to leave it there for long. You'd proably want to rebase it to a better location (not the mega merge commit), before sending the change up for pull review. For example, you can first rebase the new change `main`:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mrebase[39m [36m-r[39m [36mrwq[39m [36m-d[39m [36mmain[39m
Also rebased 1 descendant commits onto parent of rebased commit
Working copy now at: [1m[38;5;13movy[38;5;8mpxnus[39m [38;5;12m68ba[38;5;8mcc1f[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m6e4[0m[38;5;8mf5799[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Added 0 files, modified 1 files, removed 0 files
```

The `-r` option rebases only the given revision on top of the destination; it rebases all of its descendants on top of its parents. Effectively, this is similar to moving a commit to another location in the graph.

After rebasing onto `main`, you can then add `rws` as a new parent of the merge commit to keep the change applied to your working directory:

```ansi
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

This persists the change in the working directory, whilst extracting it to a standalone commit which can be sent for code review. Here's how you can create a branch and push to an upstream repository like GitHub:

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

Another possible scenario is that you've made some modifications to your working copy, and want to shift them into one the of the arms of the merge commit.

This is what the commit graph looks like after making the change:

```ansi
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
1. Rebase `ovy` onto `rwq`, then rebase the merge commit to point to `ovy` instead of `rwq`, then update the branch `test` to point to `ovy`.
2. Create a new commit after `rwq`, then squash the changes from `ovy` into it, then update the branch `test` to point to `ovy`.

The first way is similar to what's already been done above, so I'll show the second way of doing this. First, we insert a new commit after `rwq`, making sure to specify `--no-edit` to avoid checking out the changes in `rwq`:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mnew[39m [36m--after[39m [36mrwq[39m [36m--no-edit[39m
Created new commit [1m[38;5;5mlqksr[0m[38;5;8mtkk[39m [1m[38;5;4m6a3[0m[38;5;8m8dd7a[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Rebased 3 descendant commits
Working copy now at: [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;12m355e[38;5;8ma4ba[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5movy[0m[38;5;8mpxnus[39m [1m[38;5;4m27b[0m[38;5;8maf0ef[39m misc: test change
```

A new, empty commit with change ID `lqks` was created after `rwq`. Note how `lqks` was correctly inserted between `orl` and `rwq`, maintaining the ancestry of the merge commit:

```ansi
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

```ansi
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

However, there are [plans][jj-rebase-move-commits] to improve Jujutsu to make it easier to move commits around the commit graph. In the future, a command like `jj rebase -r ovy --after rwq` might be able to move the commit whilst maintaining its chnage ID.

## Removing parents

Again, we can use `jj rebase` (and a small change to the revset) to remove parents from a merge commit:

```ansi
[0;1m[32m❯[0m [34mjj[39m [36mrebase[39m [36m-s[39m [36morl[39m [36m-d[39m [33m"all:orl- ~ qkl"[39m
Rebased 2 commits
Working copy now at: [1m[38;5;13muyl[38;5;8mlouwm[39m [38;5;12m521[38;5;8me9749[39m [38;5;10m(empty)[39m [38;5;10m(no description set)[0m
Parent commit      : [1m[38;5;5morl[0m[38;5;8mlnptq[39m [1m[38;5;4m090[0m[38;5;8mffb0d[39m [38;5;2m(empty)[39m [38;5;2m(no description set)[39m
Added 0 files, modified 9 files, removed 0 files
```

Previously, when adding new parents, we've specified the destinations using the flags `-d "all:orl-" -d NEW_PARENT_ID`. Now, we're specifying the destinations using `-d "all:orl- ~ qkl"`. The new argument for the destination highlights more of the revset language, in particular the set exclusion operator. As before, `orl-` evaluates to the set of all parents of `orl`, but `~ qkl` now excludes `qkl` from that set.

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

## Conclusion

I've now shown the various ways you can use Jujutsu to manipulate merge commits, by adding and removing parents using the `jj rebase` command.

Working on multiple separate branches of code at the same time can be really powerful. Personally, I use this workflow all the time to avoid having to switch branches. This is especially convenient when working on small bugfixes where it's definitely easier to just work in the current directory.

Merging multiple branches together also allows you to very simply test out various features at the same time, without having to wait for all of them to be merged into the main branch. In fact, I use this to build a custom `jj` binary which contain various features that are still in development, but are functional.

Even if you aren't convinced about switching to Jujutsu, I think this workflow is still valuable. In fact, a new Git client [GitButler][gitbutler] was recently launched with a similar end product: making it easy to activate different "virtual branches" in your working directory. Otherwise, alternative tools like [git-branchless][git-branchless] might allow you to do something similar.

If you are intrigued by Jujutsu, do check out the [introduction][jj-intro] and [tutorial][jj-tutorial]. I'd also recommend [Chris's article][chris-krycho-jj-init] and [video series][chris-krycho-video-series].


[^1]: Hmmm, maybe *The Austin™ Mega Merge Strategy<sup>®</sup>* is the better name after all...
[^2]: Actually, you **can** have change IDs associated with multiple commits, but that's out of the scope of this article.

[chris-krycho-jj-init]: https://v5.chriskrycho.com/essays/jj-init/
[jj]: https://martinvonz.github.io/jj/
[aseipp]: https://www.austinseipp.com/
[jj-repo]: https://github.com/martinvonz/jj
[jj-revsets]: https://martinvonz.github.io/jj/latest/revsets/
[mercurial]: https://www.mercurial-scm.org/
[jj-conflicts]: https://martinvonz.github.io/jj/latest/conflicts/
[jj-undo]: https://martinvonz.github.io/jj/latest/operation-log/
[jj-rebase-move-commits]: https://github.com/martinvonz/jj/issues/1188
[gitbutler]: https://gitbutler.com/
[git-branchless]: https://github.com/arxanas/git-branchless
[jj-intro]: https://github.com/martinvonz/jj#introduction
[jj-tutorial]: https://martinvonz.github.io/jj/v0.16.0/tutorial/
[chris-krycho-video-series]: https://www.youtube.com/playlist?list=PLelyiwKWHHAq01Pvmpf6x7J0y-yQpmtxp
