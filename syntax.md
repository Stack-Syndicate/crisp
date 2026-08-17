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
(def buzz
  (fn [name:str age:i32] -> str
    (+ name age)))
(defn buzz [name:str age:i32] -> str
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

Protocols

```crisp
(defp Printable [name:str, dist:i32]
  (defn get_dist[p:@Printable]->i32
    (p.dist)
  )
  (defn get_addr[p:@Printable]->str
    (p.name)
  )
)
```

Macros

```
(defn Foo[name:code, place:code] -> code
  #($name $place)
)
```
