<?xml version="1.0"?> 

<!-- mode-onnomatch010 -->
<!-- Michael Kay -->
<!-- on-no-match=various, PI nodes --> 

<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"
  xmlns:xs="http://www.w3.org/2001/XMLSchema"
  exclude-result-prefixes="xs">

  <xsl:mode name="c" on-no-match="shallow-copy"/> 
  <xsl:mode name="d" on-no-match="shallow-skip"/>
  <xsl:mode name="s" on-no-match="text-only-copy"/>
  
  <xsl:variable name="temp" as="processing-instruction()">
    <xsl:processing-instruction name="nnn">abracadabra</xsl:processing-instruction>
  </xsl:variable>
  
  <xsl:template name="main">
    <out>
      <c><xsl:apply-templates select="$temp" mode="c"/></c>
      <d><xsl:apply-templates select="$temp" mode="d"/></d>
      <s><xsl:apply-templates select="$temp" mode="s"/></s>
    </out>
  </xsl:template>
  
    
</xsl:stylesheet>