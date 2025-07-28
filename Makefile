CXX = g++
CXXFLAGS = -std=c++17 -Wall -Wextra -O2
TARGET = peachc
SRCDIR = src
GENDIR = src/gen
OBJDIR = obj
BINDIR = .

# Source files
MAIN_SOURCES = $(wildcard $(SRCDIR)/*.cpp)
GEN_SOURCES = $(wildcard $(GENDIR)/*.cpp)
ALL_SOURCES = $(MAIN_SOURCES) $(GEN_SOURCES)

# Object files
MAIN_OBJECTS = $(MAIN_SOURCES:$(SRCDIR)/%.cpp=$(OBJDIR)/%.o)
GEN_OBJECTS = $(GEN_SOURCES:$(GENDIR)/%.cpp=$(OBJDIR)/gen/%.o)
ALL_OBJECTS = $(MAIN_OBJECTS) $(GEN_OBJECTS)

# Include directories
INCLUDES = -I$(SRCDIR) -I$(GENDIR)

all: $(BINDIR)/$(TARGET)

$(BINDIR)/$(TARGET): $(ALL_OBJECTS)
	$(CXX) $(CXXFLAGS) -o $@ $^

$(OBJDIR)/%.o: $(SRCDIR)/%.cpp | $(OBJDIR)
	$(CXX) $(CXXFLAGS) $(INCLUDES) -c -o $@ $<

$(OBJDIR)/gen/%.o: $(GENDIR)/%.cpp | $(OBJDIR)/gen
	$(CXX) $(CXXFLAGS) $(INCLUDES) -c -o $@ $<

$(OBJDIR):
	mkdir -p $(OBJDIR)

$(OBJDIR)/gen:
	mkdir -p $(OBJDIR)/gen

clean:
	rm -rf $(OBJDIR) $(BINDIR)/$(TARGET)

.PHONY: all clean