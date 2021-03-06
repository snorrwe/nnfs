import progressbar
from uuid import uuid4

from .layer import InputLayer


class Model:
    def __init__(self):
        self.layers = []
        self.prevlayer = {}
        self.nextlayer = {}
        self.trainable = []
        self.baked = False

    def add(self, layer):
        if not hasattr(layer, "id"):
            layer.id = uuid4()
        self.layers.append(layer)
        if hasattr(layer, "weights"):
            self.trainable.append(layer)

    def set(self, *, loss, optimizer, accuracy, input_layer=None):
        self.loss = loss
        self.optimizer = optimizer
        self.input_layer = input_layer if input_layer is not None else InputLayer()
        self.accuracy = accuracy
        if not hasattr(self.input_layer, "id"):
            self.input_layer.id = uuid4()

    def bake(self):
        """
        prepares the previously added layers for execution
        """
        if not self.layers:
            return

        self.loss.trainable_layers = self.trainable

        count = len(self.layers)
        lastid = self.layers[0].id
        self.prevlayer[lastid] = self.input_layer
        for i in range(1, count):
            l = self.layers[i]
            self.nextlayer[lastid] = l
            lastid = l.id
            self.prevlayer[lastid] = self.layers[i - 1]
        self.nextlayer[lastid] = self.loss
        self.output_activation = self.layers[i]
        self.loss.trainable_layers = self.trainable
        self.baked = True

    def forward(self, X):
        assert self.baked

        self.input_layer.forward(X)
        for l in (
            l for l in self.layers if not hasattr(l, "training_only") or l.training_only
        ):
            l.forward(self.prevlayer[l.id].output)
        return l.output

    def forward_train(self, X):
        """
        `forward` to be used in training
        """
        assert self.baked

        self.input_layer.forward(X)
        for l in self.layers:
            l.forward(self.prevlayer[l.id].output)
        return l.output

    def backward(self, output, y):
        assert self.baked

        self.loss.backward(output, y)
        for l in reversed(self.layers):
            l.backward(self.nextlayer[l.id].dinputs)

    def train(self, X, y, *, epochs=1, print_every=1, validation=None):
        assert self.baked

        last = -1.0

        self.accuracy.init(y)
        for epoch in progressbar.progressbar(range(epochs + 1), redirect_stdout=True):
            output = self.forward_train(X)
            data_loss, reg_loss = self.loss.calculate(
                output, y, include_regularization=True
            )
            loss = data_loss + reg_loss

            self.backward(output, y)

            self.optimizer.pre_update()
            for l in self.trainable:
                self.optimizer.update_params(l)

            if print_every and epoch % print_every == 0:
                #  assert data_loss != last, "something's wrong i can feel it"
                last = loss
                lr = self.optimizer.lr[0]
                pred = self.output_activation.predictions()
                acc = self.accuracy.calculate(pred, y)
                print(
                    f"Epoch {epoch:05} Loss: {loss:.16f} Accuracy: {acc:.16f} Learning Rate: {lr:.16f}"
                )
                if validation is not None:
                    X_val, y_val = validation
                    output_val = self.forward(X_val)
                    data_loss, _ = self.loss.calculate(output_val, y_val)
                    pred = self.output_activation.predictions()
                    accuracy = self.accuracy.calculate(pred, y_val)
                    print(
                        f"Validation  Loss: {data_loss:.16f} Accuracy: {accuracy:.16f}"
                    )
