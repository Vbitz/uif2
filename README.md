# UI Framework 2

A thin client solution for building UIs.

## Motivation

Writing a user interface in modern languages is kind of annoying. Hence there are 100s of UI libraries. UIF2 is my attempt to add to that with a UI library that I enjoy using.

UIF2 is a "thin client". As I was looking at different UI libraries there's an abundance of good solutions but each is focused on a single language.

- In C++ you have plenty of choice. GTK+ and QT are the obvious cross-platform choices there. ImGUI is a little less obvious but it's a great solution for smaller experimental applications and development UIs.
- In C# you have Windows Forms and WPF.
- In Java you have Swing and JavaFX among others.
- In Swift you have SwiftUI, Cocoa, and UIKit.
- In Rust you have eGUI.

As soon as you move of that language the developer experience working with the UI library starts to fall down.

The goal of UIF2 is to take these UI libraries and make them available to everyone over a simple network boundary.

UIF was an earlier project of mine using Direct2D and gRPC. It was handy for prototyping until I started wanting more mature UI controls.

UIF2 solves that by switching to a cross-platform library (eGUI) with mature UI controls.

## Architecture

UIF2 internally represents the UI as a element tree (like HTML and most other UIs). This element tree can be effectively manipulated by another process with a simple JSON based API.
