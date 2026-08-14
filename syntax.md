# Crisp Syntax Notes

Variables

```crisp
(def x 10)

(def y "hello")

(def z (+ 10 10))

(def a:i32 10)
```

Functions

```crisp
(def foo
  (fn [name:str age:i32] -> str
    (+ name age)))
(defn foo [name:str age:i32] -> str
  (+ name age))
```

Maps
Definition

```crisp
(defm Bar [address:str distance:f32])
(defm Foo (+ Bar [people:i32]))
```

Instantiation

```crisp
(def bar {:address "wayburg" :distance 5.0})
(def bar (Bar "wayburg" 5.0))
```

Defining protocols ???

```crisp
(defp Printable [
  get_dist:fn->i32
  get_addr:fn->str
])
(defn get_dist[bar:Bar]->i32 ())
(defn get_addr[bar:Bar]->str ())
```
