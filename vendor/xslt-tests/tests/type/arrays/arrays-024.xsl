<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" 
  xmlns:xs="http://www.w3.org/2001/XMLSchema" 
  xmlns:math="http://www.w3.org/2005/xpath-functions/math"  
   exclude-result-prefixes="xs math" version="4.0" expand-text="true">
  
  <!-- array matching in patterns -->
  
  <xsl:template match="array(xs:long)">
    <L><xsl:next-match/></L>
  </xsl:template>
  
  <xsl:template match="array(xs:integer)">
    <I><xsl:next-match/></I>
  </xsl:template>
  
  <xsl:template match="array(xs:decimal)">
    <D><xsl:next-match/></D>
  </xsl:template>
    
  <xsl:template match="array(record(first, last))">
    <AT>
      <xsl:apply-templates select="?*"/>
    </AT>
  </xsl:template>
  
  <xsl:template match="record(first, last)">
    <P first="{?first}" last="{?last}"/>
  </xsl:template>
    
    <xsl:variable name="data" expand-text="no">
      [{"first":"Sue", "last":"Schreiber"},
       {"first":"Debbie", "last":"Lockett"}]
    </xsl:variable>
  
  <xsl:template name="xsl:initial-template">
    <out>
      <xsl:apply-templates select="parse-json($data)"/>
    </out>
  </xsl:template>
  
</xsl:stylesheet>