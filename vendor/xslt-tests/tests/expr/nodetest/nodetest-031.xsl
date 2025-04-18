<?xml version="1.0" encoding="UTF-8"?>
<t:transform xmlns:my="http://www.schemanodetest.example.com/ns/various"
             xmlns:t="http://www.w3.org/1999/XSL/Transform"
             exclude-result-prefixes="my"
             version="2.0">
<!-- Purpose: Test schema-attribute($name) in the middle of a path expression.-->

   <t:import-schema namespace="http://www.schemanodetest.example.com/ns/various"
                    schema-location="variousTypesNodeTest.xsd"/>

   <t:template match="/">
	     <out>
         <t:value-of select="my:userNode/my:complexSimpleContentElem/schema-attribute(my:count)/self::node()/name()"/>
      </out>
   </t:template>
</t:transform>
